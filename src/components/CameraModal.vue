<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch, nextTick } from 'vue';
import { Camera, X, Image as ImageIcon } from 'lucide-vue-next';
import { commands, type CameraDevice } from '@/bindings';

const props = defineProps<{
  show: boolean;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'photo-taken', photoData: string): void;
}>();

const canvasRef = ref<HTMLCanvasElement | null>(null);
const fileInputRef = ref<HTMLInputElement | null>(null);
const error = ref<string | null>(null);
const cameras = ref<CameraDevice[]>([]);
const selectedDeviceIndex = ref<number | null>(null);
const streamActive = ref(false);

let ws: WebSocket | null = null;
let streamUrl: string | null = null;

async function loadCameras() {
  const result = await commands.listCameras();
  if (result.status === 'ok') {
    cameras.value = result.data;
    if (result.data.length > 0 && selectedDeviceIndex.value === null) {
      selectedDeviceIndex.value = result.data[0].index;
    }
  } else {
    error.value = 'Could not list cameras: ' + result.error;
  }
}

function connectStream(wsUrl: string) {
  disconnectStream();

  ws = new WebSocket(wsUrl);
  ws.binaryType = 'arraybuffer';

  ws.onmessage = (event) => {
    const blob = new Blob([event.data], { type: 'image/jpeg' });
    createImageBitmap(blob).then((bitmap) => {
      const canvas = canvasRef.value;
      if (!canvas) return;
      const ctx = canvas.getContext('2d');
      if (!ctx) return;
      // Resize canvas to match frame dimensions
      if (canvas.width !== bitmap.width || canvas.height !== bitmap.height) {
        canvas.width = bitmap.width;
        canvas.height = bitmap.height;
      }
      ctx.drawImage(bitmap, 0, 0);
      bitmap.close();
    });
  };

  ws.onerror = () => {
    error.value = 'Stream connection error';
  };

  ws.onclose = () => {
    streamActive.value = false;
  };

  ws.onopen = () => {
    streamActive.value = true;
  };
}

function disconnectStream() {
  if (ws) {
    ws.close();
    ws = null;
  }
  streamActive.value = false;
}

async function startCamera() {
  if (selectedDeviceIndex.value === null) return;
  error.value = null;
  try {
    const result = await commands.startCamera(selectedDeviceIndex.value);
    if (result.status === 'ok') {
      streamUrl = result.data;
      // Convert http URL to ws URL
      const wsUrl = streamUrl.replace(/^http/, 'ws');
      await nextTick();
      connectStream(wsUrl);
    } else {
      error.value = 'Could not start camera: ' + result.error;
    }
  } catch (e: any) {
    error.value = 'Could not start camera: ' + e.message;
  }
}

async function stopCamera() {
  disconnectStream();
  streamUrl = null;
  await commands.stopCamera();
}

async function takePhoto() {
  const result = await commands.snapPhoto();
  if (result.status === 'ok') {
    emit('photo-taken', result.data);
    emit('close');
  } else {
    error.value = 'Failed to capture photo: ' + result.error;
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

watch(() => props.show, async (newVal) => {
  if (newVal) {
    await loadCameras();
    if (selectedDeviceIndex.value !== null) {
      await startCamera();
    }
  } else {
    await stopCamera();
  }
});

watch(selectedDeviceIndex, async (newVal, oldVal) => {
  if (props.show && newVal !== null && oldVal !== null && newVal !== oldVal) {
    await startCamera();
  }
});

onMounted(async () => {
  if (props.show) {
    await loadCameras();
    if (selectedDeviceIndex.value !== null) {
      await startCamera();
    }
  }
});

onUnmounted(() => {
  disconnectStream();
  commands.stopCamera();
});
</script>

<template>
  <div v-if="show" class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black bg-opacity-75 backdrop-blur-sm">
    <div class="bg-white rounded-lg shadow-xl w-full max-w-lg overflow-hidden relative flex flex-col max-h-[90vh]">
      <!-- Header -->
      <div class="p-4 border-b border-gray-200 flex items-center justify-between bg-gray-50">
        <div class="flex-1">
             <select
                v-if="cameras.length > 0"
                v-model="selectedDeviceIndex"
                class="block w-full max-w-[200px] rounded-md border-gray-300 py-1.5 text-base leading-5 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500 sm:text-sm"
            >
                <option v-for="device in cameras" :key="device.index" :value="device.index">
                    {{ device.name || 'Camera ' + (device.index + 1) }}
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
        <canvas
          ref="canvasRef"
          class="w-full h-full object-contain max-h-[60vh]"
        />
        <div v-if="!streamActive && !error" class="absolute inset-0 flex items-center justify-center text-gray-400 text-sm">
          Starting camera...
        </div>
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
          :disabled="!streamActive"
        >
          <Camera class="h-8 w-8 text-blue-600" />
        </button>

        <!-- Spacer to balance layout -->
        <div class="w-12"></div>
      </div>

      <!-- Hidden file input -->
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
