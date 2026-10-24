<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { Check } from 'lucide-vue-next';
import CameraFrame from './CameraFrame.vue';
import { useCameraSettings } from '@/stores/camera';
const settings = useCameraSettings();
import { clampCrop, type Crop } from '@/lib/cameraSettings';
import anteriorMask from '@/assets/alignment-mask-anterior.svg';
import posteriorMask from '@/assets/alignment-mask-posterior.svg';
import rightLateralMask from '@/assets/alignment-mask-right-lateral.svg';
import leftLateralMask from '@/assets/alignment-mask-left-lateral.svg';

const props = defineProps<{
  show: boolean;
  imageSrc: string | null;
  initialCrop?: Crop | null;
  imageType?: string | null;
}>();
const emit = defineEmits<{ (e: 'close'): void; (e: 'save', crop: Crop): void }>();
const maskSrc = computed(() => {
  switch (props.imageType) {
    case 'anterior': return anteriorMask;
    case 'posterior': return posteriorMask;
    case 'right_lateral': return rightLateralMask;
    case 'left_lateral': return leftLateralMask;
    default: return null;
  }
});
const imgRef = ref<HTMLImageElement | null>(null);
const dimensions = ref({ width: 0, height: 0 });
const ready = ref(false);
const imageError = ref('');
const crop = ref<Crop | null>(null);
const aspect = computed(() => dimensions.value.width && dimensions.value.height ? dimensions.value.width / dimensions.value.height : 16 / 9);
const frame = computed(() => clampCrop(crop.value, aspect.value));
function loaded() {
  const img = imgRef.value;
  if (!img?.naturalWidth || !img.naturalHeight) return;
  dimensions.value = { width: img.naturalWidth, height: img.naturalHeight };
  crop.value = clampCrop(crop.value, aspect.value);
  ready.value = true;
}
function save() {
  if (!ready.value) return;
  // Only crop coordinates leave the editor. The original image is unchanged.
  emit('save', { ...frame.value });
  emit('close');
}
watch(() => [props.show, props.imageSrc] as const, () => {
  ready.value = false;
  imageError.value = '';
  dimensions.value = { width: 0, height: 0 };
  crop.value = props.initialCrop ? { ...props.initialCrop } : null;
}, { immediate: true });
</script>

<template>
  <div v-if="show" class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/90 backdrop-blur-sm" role="dialog" aria-modal="true" aria-label="Adjust image framing" @keydown.esc="$emit('close')">
    <div class="w-full max-w-4xl max-h-[90vh] overflow-y-auto flex flex-col gap-3">
      <div class="flex flex-wrap justify-between items-center gap-3 px-1">
        <h3 class="text-white font-bold text-lg">Adjust framing (2:3)</h3>
        <div class="flex gap-2">
          <button @click="$emit('close')" class="px-4 py-2 bg-gray-800 text-white rounded hover:bg-gray-700">Cancel</button>
          <button @click="save" :disabled="!ready" class="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-500 flex items-center gap-2 disabled:opacity-50"><Check class="w-4 h-4" /> Save</button>
        </div>
      </div>
      <p class="text-gray-300 text-sm">The full original image is shown. Move or resize the frame; saving changes only the framing.</p>
      <div class="w-full bg-black flex items-start justify-center overflow-hidden">
        <div class="relative flex-none overflow-hidden" :style="{ width: `min(100%, ${65 * aspect}vh)`, aspectRatio: String(aspect), transform: settings.preferences.mirrorPreviews ? 'scaleX(-1)' : undefined }">
          <img ref="imgRef" :key="imageSrc || ''" :src="imageSrc || ''" alt="Full original capture" class="absolute inset-0 block w-full h-full object-contain pointer-events-none select-none" @load="loaded" @error="imageError = 'Could not load the original image.'; ready = false" />
          <CameraFrame v-if="ready" :crop="frame" :aspect="aspect" :mirrored="settings.preferences.mirrorPreviews" editable @update="crop = $event">
            <img v-if="maskSrc" :src="maskSrc" alt="" class="absolute inset-0 w-full h-full object-cover pointer-events-none opacity-50" />
          </CameraFrame>
        </div>
      </div>
      <p v-if="imageError" role="alert" class="text-red-400">{{ imageError }}</p>
      <div v-if="ready" class="flex justify-between gap-3 text-sm text-gray-300">
        <span>Original: {{ dimensions.width }} × {{ dimensions.height }} px</span>
        <button class="text-blue-300 underline" @click="crop = clampCrop(null, aspect)">Reset framing</button>
      </div>
    </div>
  </div>
</template>
