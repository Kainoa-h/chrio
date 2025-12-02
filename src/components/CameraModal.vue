<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { Camera, X } from 'lucide-vue-next';

const props = defineProps<{
  show: boolean;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'photo-taken', photoData: string): void;
}>();

const videoRef = ref<HTMLVideoElement | null>(null);
const canvasRef = ref<HTMLCanvasElement | null>(null);
const stream = ref<MediaStream | null>(null);
const error = ref<string | null>(null);

async function startCamera() {
  try {
    stream.value = await navigator.mediaDevices.getUserMedia({ 
      video: { facingMode: 'environment' }, 
      audio: false 
    });
    if (videoRef.value) {
      videoRef.value.srcObject = stream.value;
    }
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

onMounted(() => {
  if (props.show) {
    startCamera();
  }
});

// Watch for prop changes to start/stop camera
import { watch } from 'vue';
watch(() => props.show, (newVal) => {
  if (newVal) {
    startCamera();
  } else {
    stopCamera();
  }
});

onUnmounted(() => {
  stopCamera();
});
</script>

<template>
  <div v-if="show" class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black bg-opacity-75 backdrop-blur-sm">
    <div class="bg-white rounded-lg shadow-xl w-full max-w-lg overflow-hidden relative">
      <!-- Close button -->
      <button 
        @click="$emit('close')" 
        class="absolute top-4 right-4 z-10 p-2 rounded-full bg-gray-800 text-white hover:bg-gray-700 transition-colors"
      >
        <X class="h-5 w-5" />
      </button>

      <div class="relative aspect-video bg-black flex items-center justify-center">
        <video ref="videoRef" autoplay playsinline class="w-full h-full object-cover"></video>
        <div v-if="error" class="absolute inset-0 flex items-center justify-center text-red-500 p-4 text-center">
          {{ error }}
        </div>
      </div>

      <div class="p-6 flex justify-center">
        <button 
          @click="takePhoto"
          class="h-16 w-16 rounded-full bg-white border-4 border-blue-500 flex items-center justify-center shadow-lg hover:bg-gray-100 transition-transform active:scale-95"
          :disabled="!!error"
        >
          <Camera class="h-8 w-8 text-blue-600" />
        </button>
      </div>
      
      <!-- Hidden canvas for capture -->
      <canvas ref="canvasRef" class="hidden"></canvas>
    </div>
  </div>
</template>
