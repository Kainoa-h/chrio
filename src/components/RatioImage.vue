<script setup lang="ts">
import { computed, ref, watch } from 'vue';

const props = defineProps<{
  src?: string | null;
  alt?: string;
  emptyText?: string;
  containerClass?: string;
  crop?: { x: number, y: number, width: number } | null;
}>();

const hasImage = computed(() => !!props.src);
const imgRef = ref<HTMLImageElement | null>(null);
const imgStyle = ref({});

function updateStyle() {
  if (!imgRef.value || !props.crop) {
    imgStyle.value = { width: '100%', height: '100%', objectFit: 'cover' };
    return;
  }

  const img = imgRef.value;
  const ar = img.naturalWidth / img.naturalHeight;
  const { x, y, width } = props.crop; // 0.0 - 1.0

  // Calculate styles to zoom and pan
  // Container is 2:3. Crop rect is 2:3.
  // We want Crop rect to fill Container.
  
  // Width of image relative to container width
  const wPct = (1 / width) * 100;
  
  // Left position: -x relative to crop width?
  // Shift = x * scale. scale = 1/width.
  // Left = - (x / width) * 100%
  const leftPct = - (x / width) * 100;
  
  // Top position
  // Scale applies to height too.
  // Top = - y_px * scale.
  // y_px = y * naturalHeight.
  // scale = containerWidth / (width * naturalWidth).
  // Top (px) = - y * naturalHeight * containerWidth / (width * naturalWidth)
  // Top (relative to containerWidth) = - (y/width) * (1/ar).
  // We want Top % relative to CONTAINER HEIGHT? 
  // Usually 'top' % is relative to parent height.
  // Container Height = Container Width * 1.5.
  // So Top% = (Top_px / ContainerHeight) * 100
  //         = [ - y * naturalHeight * containerWidth / (width * naturalWidth) ] / (containerWidth * 1.5) * 100
  //         = - (y / width) * (naturalHeight / naturalWidth) / 1.5 * 100
  //         = - (y / width) * (1/ar) / 1.5 * 100
  
  const topPct = - (y / width) * (1/ar) / 1.5 * 100;

  imgStyle.value = {
    position: 'absolute',
    width: `${wPct}%`,
    maxWidth: 'none', // Allow overflow
    height: 'auto',
    left: `${leftPct}%`,
    top: `${topPct}%`,
  };
}

watch(() => [props.src, props.crop], () => {
    // If no crop, reset immediately to cover
    if (!props.crop) {
        imgStyle.value = { width: '100%', height: '100%', objectFit: 'cover' };
    } else if (imgRef.value?.complete) {
        updateStyle();
    }
});
</script>

<template>
  <div 
    class="relative overflow-hidden bg-gray-900 rounded-lg border border-gray-800 flex items-center justify-center"
    :class="containerClass"
    style="aspect-ratio: 2 / 3;"
  >
    <img 
      v-if="hasImage" 
      ref="imgRef"
      :src="src!" 
      :alt="alt || 'Image'" 
      :style="imgStyle"
      class="block"
      @load="updateStyle"
    />
    <div v-else class="text-gray-500 flex flex-col items-center p-4 text-center">
      <slot name="empty">
        <span class="text-sm">{{ emptyText || 'No Image' }}</span>
      </slot>
    </div>
    
    <div class="absolute inset-0">
       <slot name="overlay"></slot>
    </div>
  </div>
</template>
