<script setup lang="ts">
import { computed } from 'vue';

const props = defineProps<{
  src?: string | null;
  alt?: string;
  emptyText?: string;
  containerClass?: string;
}>();

const hasImage = computed(() => !!props.src);
</script>

<template>
  <div 
    class="relative overflow-hidden bg-gray-900 rounded-lg border border-gray-800 flex items-center justify-center"
    :class="containerClass"
    style="aspect-ratio: 2 / 3;"
  >
    <img 
      v-if="hasImage" 
      :src="src!" 
      :alt="alt || 'Image'" 
      class="w-full h-full object-cover"
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
