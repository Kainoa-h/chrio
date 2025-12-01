<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { Check } from 'lucide-vue-next';
import anteriorMask from '@/assets/alignment-mask-anterior.svg';
import posteriorMask from '@/assets/alignment-mask-posterior.svg';
import rightLateralMask from '@/assets/alignment-mask-right-lateral.svg';
import leftLateralMask from '@/assets/alignment-mask-left-lateral.svg';

const props = defineProps<{
  show: boolean;
  imageSrc: string | null;
  initialCrop?: { x: number, y: number, width: number } | null;
  imageType?: string | null;
}>();

const maskSrc = computed(() => {
  switch (props.imageType) {
    case 'anterior': return anteriorMask;
    case 'posterior': return posteriorMask;
    case 'right_lateral': return rightLateralMask;
    case 'left_lateral': return leftLateralMask;
    default: return null;
  }
});

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'save', cropData: { x: number, y: number, width: number }): void;
}>();

const containerRef = ref<HTMLElement | null>(null);
const cropBoxRef = ref<HTMLElement | null>(null);

// Crop state (percentage 0-100)
const crop = ref({ x: 10, y: 10, width: 50 }); // Initial crop

// Aspect ratio 2:3 => width:height = 2:3 => height = width * 1.5
// In percentage relative to container width/height?
// Let's stick to percentage of IMAGE WIDTH for 'width' and 'x'.
// For 'y', it is percentage of IMAGE HEIGHT? Or also relative to width?
// To keep it simple: standardized coordinates relative to the image.
// x, y, width are fractions (0.0 to 1.0) of the image dimensions.
// But aspect ratio constraints depend on actual image aspect ratio.

// Let's use pixel logic for interaction, then normalize on save.
const imgRef = ref<HTMLImageElement | null>(null);

// Dragging state
const isDragging = ref(false);
const isResizing = ref(false);
const startX = ref(0);
const startY = ref(0);
const startCropX = ref(0);
const startCropY = ref(0);
const startCropWidth = ref(0);

function onMouseDown(e: MouseEvent, action: 'drag' | 'resize') {
  if (!imgRef.value) return;
  e.preventDefault();
  isDragging.value = action === 'drag';
  isResizing.value = action === 'resize';
  startX.value = e.clientX;
  startY.value = e.clientY;
  startCropX.value = crop.value.x;
  startCropY.value = crop.value.y;
  startCropWidth.value = crop.value.width;

  window.addEventListener('mousemove', onMouseMove);
  window.addEventListener('mouseup', onMouseUp);
}

function onMouseMove(e: MouseEvent) {
  if (!imgRef.value || !containerRef.value) return;
  
  // Calculate image aspect ratio
  const imgAspect = imgRef.value.naturalWidth / imgRef.value.naturalHeight;
  
  // Wait, object-fit: contain means image might not fill container.
  // We need the actual rendered dimensions of the image.
  // For simplicity, let's assume the cropper works on the visible image.
  // We will use `imgRef` directly.
  const imgRect = imgRef.value.getBoundingClientRect();
  
  const deltaX = (e.clientX - startX.value);
  const deltaY = (e.clientY - startY.value);

  // Convert delta pixels to percentage of displayed image width/height
  const deltaX_pct = (deltaX / imgRect.width) * 100;
  const deltaY_pct = (deltaY / imgRect.height) * 100;

  if (isDragging.value) {
    let newX = startCropX.value + deltaX_pct;
    let newY = startCropY.value + deltaY_pct;

    // This math is getting messy. Let's simplify:
    // crop.width is % of image width.
    // crop.height (pixels) = (crop.width_pct / 100 * imgWidth) * 1.5
    // crop.height_pct = crop.height_px / imgHeight * 100
    //                 = (crop.width_pct/100 * imgWidth * 1.5 / imgHeight) * 100
    //                 = crop.width_pct * 1.5 * (imgWidth / imgHeight)
    //                 = crop.width_pct * 1.5 * imgAspect
    
    const h_pct = crop.value.width * 1.5 * imgAspect;

    // Clamp
    newX = Math.max(0, Math.min(newX, 100 - crop.value.width));
    newY = Math.max(0, Math.min(newY, 100 - h_pct));

    crop.value.x = newX;
    crop.value.y = newY;
  }

  if (isResizing.value) {
    let newWidth = startCropWidth.value + deltaX_pct;
    
    // Constraint: Width must not exceed 100% - x, and Height must not exceed 100% - y
    // h = w * 1.5 * aspect
    // w <= (100 - y) / (1.5 * aspect)
    
    const aspectFactor = 1.5 * imgAspect;
    const maxWidthByX = 100 - crop.value.x;
    const maxWidthByY = (100 - crop.value.y) / aspectFactor;
    
    newWidth = Math.min(newWidth, maxWidthByX, maxWidthByY);
    newWidth = Math.max(10, newWidth); // Min width 10%

    crop.value.width = newWidth;
  }
}

function onMouseUp() {
  isDragging.value = false;
  isResizing.value = false;
  window.removeEventListener('mousemove', onMouseMove);
  window.removeEventListener('mouseup', onMouseUp);
}

// We need aspect ratio for proper rendering of the box height in %
const imgAspectRatio = ref(1);
function onImgLoad() {
  if (imgRef.value) {
    imgAspectRatio.value = imgRef.value.naturalWidth / imgRef.value.naturalHeight;
  }
}

function save() {
  emit('save', { 
    x: crop.value.x / 100, 
    y: crop.value.y / 100, 
    width: crop.value.width / 100 
  });
  emit('close');
}

watch(() => props.show, (newVal) => {
  if (newVal) {
    if (props.initialCrop) {
      crop.value = {
        x: props.initialCrop.x * 100,
        y: props.initialCrop.y * 100,
        width: props.initialCrop.width * 100
      };
    } else {
      crop.value = { x: 10, y: 10, width: 50 };
    }
  }
});
</script>

<template>
  <div v-if="show" class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black bg-opacity-90 backdrop-blur-sm">
    <div class="relative w-full max-w-4xl h-full max-h-[90vh] flex flex-col">
      
      <!-- Toolbar -->
      <div class="flex justify-between items-center mb-4 px-4">
        <h3 class="text-white font-bold text-lg">Crop Image (2:3)</h3>
        <div class="flex gap-2">
          <button @click="$emit('close')" class="px-4 py-2 bg-gray-800 text-white rounded hover:bg-gray-700">Cancel</button>
          <button @click="save" class="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-500 flex items-center gap-2">
            <Check class="w-4 h-4" /> Save
          </button>
        </div>
      </div>

      <!-- Editor Area -->
      <div class="flex-1 relative bg-black flex items-center justify-center overflow-hidden" ref="containerRef">
        <div class="relative inline-block">
          <img 
            ref="imgRef"
            :src="imageSrc || ''" 
            class="max-w-full max-h-[80vh] object-contain block pointer-events-none select-none"
            @load="onImgLoad"
          />
          
          <!-- Overlay -->
          <div class="absolute inset-0 bg-black bg-opacity-50"></div>

          <!-- Crop Box (Clear area) -->
          <!-- Using a simpler approach: A box on top that shows the image, with rest obscured? 
               Or simpler: A div with border that allows dragging. 
               To show the "clear" part, we can use box-shadow with huge spread, or clip-path.
               Box-shadow is easiest.
          -->
          <div 
            ref="cropBoxRef"
            class="absolute border-2 border-white cursor-move"
            :style="{
              left: `${crop.x}%`,
              top: `${crop.y}%`,
              width: `${crop.width}%`,
              height: `${crop.width * 1.5 * imgAspectRatio}%`,
              boxShadow: '0 0 0 9999px rgba(0, 0, 0, 0.5)'
            }"
            @mousedown.stop="onMouseDown($event, 'drag')"
          >
            <!-- Resize Handle -->
            <div 
              class="absolute bottom-0 right-0 w-4 h-4 bg-blue-500 cursor-se-resize"
              @mousedown.stop="onMouseDown($event, 'resize')"
            ></div>
            
            <!-- Grid lines -->
            <div class="absolute inset-0 pointer-events-none border border-white border-opacity-20 grid grid-cols-3 grid-rows-3">
               <!-- We can use background image or separate divs for grid, but keep it simple -->
            </div>

            <!-- Alignment Mask Overlay -->
            <img 
              v-if="maskSrc"
              :src="maskSrc"
              class="absolute inset-0 w-full h-full object-cover pointer-events-none opacity-50"
            />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
