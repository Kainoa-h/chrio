<script setup lang="ts">
import { ref, computed } from 'vue';
import { X, ChevronsLeftRight } from 'lucide-vue-next';
import RatioImage from './RatioImage.vue';

const props = defineProps<{
  show: boolean;
  title: string;
  image1Src: string | null;
  image2Src: string | null;
  crop1: { x: number, y: number, width: number } | null;
  crop2: { x: number, y: number, width: number } | null;
  label1?: string;
  label2?: string;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
}>();

const sliderPosition = ref(50); // 0 to 100
const containerRef = ref<HTMLElement | null>(null);
const isDragging = ref(false);

function onMouseDown(e: MouseEvent) {
  isDragging.value = true;
  updateSliderPosition(e);
  window.addEventListener('mousemove', onMouseMove);
  window.addEventListener('mouseup', onMouseUp);
}

function onMouseMove(e: MouseEvent) {
  if (isDragging.value) {
    updateSliderPosition(e);
  }
}

function onMouseUp() {
  isDragging.value = false;
  window.removeEventListener('mousemove', onMouseMove);
  window.removeEventListener('mouseup', onMouseUp);
}

function updateSliderPosition(e: MouseEvent) {
  if (!containerRef.value) return;
  const rect = containerRef.value.getBoundingClientRect();
  const x = e.clientX - rect.left;
  const percentage = Math.max(0, Math.min(100, (x / rect.width) * 100));
  sliderPosition.value = percentage;
}

// Clip path for the top image (Image 1 - Before/Left)
// We want to show Image 1 on the LEFT of the slider.
// So we clip the RIGHT side of Image 1.
// inset(top right bottom left)
const image1Style = computed(() => ({
  clipPath: `inset(0 ${100 - sliderPosition.value}% 0 0)`
}));

</script>

<template>
  <div v-if="show" class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black bg-opacity-90 backdrop-blur-sm">
    <div class="relative w-full max-w-4xl h-[90vh] flex flex-col">
      
      <!-- Header -->
      <div class="flex justify-between items-center mb-4 px-4">
        <h3 class="text-white font-bold text-lg">Compare {{ title }}</h3>
        <button @click="$emit('close')" class="p-2 bg-gray-800 text-white rounded-full hover:bg-gray-700">
          <X class="w-5 h-5" />
        </button>
      </div>

      <!-- Comparison Area -->
      <div class="flex-1 flex items-center justify-center overflow-hidden p-4">
        <div 
          ref="containerRef"
          class="relative w-full h-full max-w-2xl select-none"
          style="aspect-ratio: 2/3;"
          @mousedown="onMouseDown"
        >
          <!-- Image 2 (After/Right) - Background Layer -->
          <div class="absolute inset-0">
            <RatioImage 
              :src="image2Src" 
              :crop="crop2" 
              empty-text="No Image"
              container-class="w-full h-full"
            />
            <!-- Label -->
            <div class="absolute top-4 right-4 bg-black bg-opacity-50 text-white px-2 py-1 rounded text-sm pointer-events-none">
              {{ label2 || 'After' }}
            </div>
          </div>

          <!-- Image 1 (Before/Left) - Top Layer with Clip Path -->
          <div 
            class="absolute inset-0"
            :style="image1Style"
          >
            <RatioImage 
              :src="image1Src" 
              :crop="crop1" 
              empty-text="No Image"
              container-class="w-full h-full"
            />
            <!-- Label -->
            <div class="absolute top-4 left-4 bg-black bg-opacity-50 text-white px-2 py-1 rounded text-sm pointer-events-none">
              {{ label1 || 'Before' }}
            </div>
          </div>

          <!-- Slider Handle -->
          <div 
            class="absolute top-0 bottom-0 w-1 bg-white cursor-ew-resize flex items-center justify-center shadow-lg z-10"
            :style="{ left: `${sliderPosition}%` }"
          >
            <div class="w-8 h-8 bg-white rounded-full flex items-center justify-center shadow-md text-gray-800">
              <ChevronsLeftRight class="w-5 h-5" />
            </div>
          </div>

        </div>
      </div>
    </div>
  </div>
</template>
