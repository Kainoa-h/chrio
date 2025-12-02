<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue';
import { Camera, X, Image as ImageIcon } from 'lucide-vue-next';

const props = defineProps<{
  show: boolean;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'photo-taken', photoData: string): void;
}>();

const videoRef = ref<HTMLVideoElement | null>(null);
const canvasRef = ref<HTMLCanvasElement | null>(null);
const fileInputRef = ref<HTMLInputElement | null>(null);
const stream = ref<MediaStream | null>(null);
const error = ref<string | null>(null);
const videoDevices = ref<MediaDeviceInfo[]>([]);
const selectedDeviceId = ref<string>('');

async function getDevices() {
  try {
    const devices = await navigator.mediaDevices.enumerateDevices();
    videoDevices.value = devices.filter(device => device.kind === 'videoinput');
    
    // If we have a stream, try to sync selectedDeviceId with currently active track if not set
    if (stream.value && !selectedDeviceId.value) {
        const track = stream.value.getVideoTracks()[0];
        if (track) {
            const settings = track.getSettings();
            if (settings.deviceId) {
                selectedDeviceId.value = settings.deviceId;
            }
        }
    }
  } catch (e) {
    console.error("Error getting devices:", e);
  }
}

async function startCamera() {
  stopCamera();
  try {
    const constraints: MediaStreamConstraints = {
      audio: false,
      video: selectedDeviceId.value 
        ? { deviceId: { exact: selectedDeviceId.value } } 
        : { facingMode: 'environment' }
    };
    
    stream.value = await navigator.mediaDevices.getUserMedia(constraints);
    if (videoRef.value) {
      videoRef.value.srcObject = stream.value;
    }
    error.value = null;
    
    // Update device list after getting permission, as labels might now be available
    await getDevices();
  } catch (e: any) {
    error.value = "Could not access camera: " + e.message;
    console.error("Camera error:", e);
  }
}

function stopCamera() {
  if (stream.value) {
    stream.value.getTracks().forEach(track => track.stop());
    stream.value = null;
  }
}

function takePhoto() {
  if (!videoRef.value || !canvasRef.value) return;

  const video = videoRef.value;
  const canvas = canvasRef.value;
  const context = canvas.getContext('2d');

  if (context) {
    canvas.width = video.videoWidth;
    canvas.height = video.videoHeight;
    context.drawImage(video, 0, 0, canvas.width, canvas.height);
    const photoData = canvas.toDataURL('image/jpeg');
    emit('photo-taken', photoData);
    emit('close');
  }
}

function triggerFileUpload() {
  fileInputRef.value?.click();
}

function handleFileChange(event: Event) {
  const input = event.target as HTMLInputElement;
  if (input.files && input.files[0]) {
    const file = input.files[0];
    const reader = new FileReader();
    reader.onload = (e) => {
      const result = e.target?.result as string;
      if (result) {
        emit('photo-taken', result);
        emit('close');
      }
    };
    reader.readAsDataURL(file);
  }
}

onMounted(() => {
  if (props.show) {
    startCamera();
  }
});

// Watch for prop changes to start/stop camera
watch(() => props.show, (newVal) => {
  if (newVal) {
    startCamera();
  } else {
    stopCamera();
  }
});

// Watch for camera selection changes
watch(selectedDeviceId, () => {
    if (props.show && stream.value) {
        // Only restart if we are already showing and have a stream
        // (avoids triggering on initial mount before permission)
        // Check if the current stream device ID is different to avoid redundant restart
        const track = stream.value.getVideoTracks()[0];
        const currentId = track?.getSettings().deviceId;
        if (currentId !== selectedDeviceId.value) {
             startCamera();
        }
    }
});

onUnmounted(() => {
  stopCamera();
});
</script>

<template>
  <div v-if="show" class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black bg-opacity-75 backdrop-blur-sm">
    <div class="bg-white rounded-lg shadow-xl w-full max-w-lg overflow-hidden relative flex flex-col max-h-[90vh]">
      <!-- Header -->
      <div class="p-4 border-b border-gray-200 flex items-center justify-between bg-gray-50">
        <div class="flex-1">
             <select 
                v-if="videoDevices.length > 0" 
                v-model="selectedDeviceId" 
                class="block w-full max-w-[200px] rounded-md border-gray-300 py-1.5 text-base leading-5 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500 sm:text-sm"
            >
                <option v-for="device in videoDevices" :key="device.deviceId" :value="device.deviceId">
                    {{ device.label || 'Camera ' + (videoDevices.indexOf(device) + 1) }}
                </option>
            </select>
            <span v-else class="font-semibold text-gray-700">Take Photo</span>
        </div>
        <button 
            @click="$emit('close')" 
            class="p-2 rounded-full text-gray-500 hover:bg-gray-200 transition-colors"
        >
            <X class="h-5 w-5" />
        </button>
      </div>

      <div class="relative bg-black flex items-center justify-center flex-grow min-h-[300px]">
        <video ref="videoRef" autoplay playsinline class="w-full h-full object-contain max-h-[60vh]"></video>
        <div v-if="error" class="absolute inset-0 flex items-center justify-center text-red-500 p-4 text-center">
          {{ error }}
        </div>
      </div>

      <div class="p-6 flex items-center justify-center gap-8 bg-gray-50 border-t border-gray-200">
        <button 
            @click="triggerFileUpload"
            class="p-3 rounded-full text-gray-600 hover:bg-gray-200 transition-colors"
            title="Upload from device"
        >
            <ImageIcon class="h-6 w-6" />
        </button>
        
        <button 
          @click="takePhoto"
          class="h-16 w-16 rounded-full bg-white border-4 border-blue-500 flex items-center justify-center shadow-lg hover:bg-gray-100 transition-transform active:scale-95"
          :disabled="!!error"
        >
          <Camera class="h-8 w-8 text-blue-600" />
        </button>

        <!-- Spacer to balance layout -->
        <div class="w-12"></div>
      </div>
      
      <!-- Hidden canvas & file input -->
      <canvas ref="canvasRef" class="hidden"></canvas>
      <input 
        type="file" 
        ref="fileInputRef" 
        accept="image/*" 
        class="hidden" 
        @change="handleFileChange"
      >
    </div>
  </div>
</template>
