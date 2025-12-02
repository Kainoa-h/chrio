<script setup lang="ts">
import { X, Crop } from 'lucide-vue-next';
import RatioImage from '@/components/RatioImage.vue';
import { onMounted, onUnmounted } from 'vue';

defineProps<{
  show: boolean;
  imageSrc: string | null;
  crop: any;
  canCrop?: boolean;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'crop'): void;
}>();

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    emit('close');
  }
}

onMounted(() => window.addEventListener('keydown', handleKeydown));
onUnmounted(() => window.removeEventListener('keydown', handleKeydown));
</script>

<template>
  <div v-if="show" class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black bg-opacity-90 backdrop-blur-sm" @click="$emit('close')">
    <button 
        @click="$emit('close')" 
        class="absolute top-4 right-4 z-10 p-2 rounded-full bg-white/10 text-white hover:bg-white/20 transition-colors"
    >
        <X class="h-6 w-6" />
    </button>

    <div class="relative flex flex-col items-center justify-center" @click.stop>
       <!-- Wrapper to constrain size while respecting aspect ratio -->
       <!-- h-[85vh] sets height. max-w-[90vw] ensures width doesn't overflow. aspect-[2/3] maintains ratio. -->
       <RatioImage 
          :src="imageSrc" 
          :crop="crop"
          container-class="h-[85vh] w-auto aspect-[2/3] shadow-2xl rounded-lg max-w-[90vw]"
        >
            <template #overlay>
                 <div v-if="canCrop" class="absolute bottom-4 right-4 opacity-0 hover:opacity-100 transition-opacity">
                     <button 
                        @click.stop="emit('crop')"
                        class="p-2 bg-gray-800/80 text-white rounded-full hover:bg-gray-700 transition-colors"
                        title="Crop Image"
                    >
                        <Crop class="h-5 w-5" />
                    </button>
                 </div>
            </template>
        </RatioImage>
    </div>
  </div>
</template>
