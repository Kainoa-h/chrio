<script setup lang="ts">
import { onBeforeUnmount } from 'vue';
import { clampCrop, type Crop } from '@/lib/cameraSettings';
const props = defineProps<{ crop: Crop; aspect: number; editable: boolean }>();
const emit = defineEmits<{ (e: 'update', value: Crop): void }>();
let cleanup = () => {};
function drag(event: PointerEvent, resize: boolean) {
  if (!props.editable) return;
  event.preventDefault();
  cleanup();
  const parent = (event.currentTarget as HTMLElement).closest('[data-camera-frame]')?.parentElement;
  if (!parent) return;
  const rect = parent.getBoundingClientRect();
  const initial = { ...props.crop };
  const move = (e: PointerEvent) => {
    const dx = (e.clientX - event.clientX) / rect.width;
    const dy = (e.clientY - event.clientY) / rect.height;
    if (resize) update({ ...initial, width: initial.width + dx });
    else emit('update', clampCrop({ ...initial, x: initial.x + dx, y: initial.y + dy }, props.aspect));
  };
  cleanup = () => { window.removeEventListener('pointermove', move); window.removeEventListener('pointerup', cleanup); window.removeEventListener('pointercancel', cleanup); };
  window.addEventListener('pointermove', move);
  window.addEventListener('pointerup', cleanup);
  window.addEventListener('pointercancel', cleanup);
}
function update(value: Crop) {
  value.width = Math.min(value.width, 1 - Math.max(0, value.x), (1 - Math.max(0, value.y)) / (1.5 * props.aspect));
  emit('update', clampCrop(value, props.aspect));
}
function key(event: KeyboardEvent) {
  if (!props.editable || !['ArrowLeft', 'ArrowRight', 'ArrowUp', 'ArrowDown'].includes(event.key)) return;
  event.preventDefault();
  const step = event.shiftKey ? 0.025 : 0.005;
  const c = { ...props.crop };
  if (event.altKey) c.width += ['ArrowRight', 'ArrowUp'].includes(event.key) ? step : -step;
  else if (event.key === 'ArrowLeft') c.x -= step;
  else if (event.key === 'ArrowRight') c.x += step;
  else if (event.key === 'ArrowUp') c.y -= step;
  else c.y += step;
  emit('update', clampCrop(c, props.aspect));
}
onBeforeUnmount(() => cleanup());
</script>

<template>
  <div data-camera-frame class="absolute border-2 border-white touch-none" :class="editable ? 'cursor-move' : 'pointer-events-none'"
    :style="{ left: `${crop.x * 100}%`, top: `${crop.y * 100}%`, width: `${crop.width * 100}%`, height: `${crop.width * 1.5 * aspect * 100}%`, boxShadow: '0 0 0 9999px rgb(0 0 0 / 50%)' }"
    :tabindex="editable ? 0 : undefined" role="group" aria-label="Camera frame. Arrow keys move; Alt and arrow keys resize. Shift makes larger adjustments."
    @pointerdown.stop="drag($event, false)" @keydown="key">
    <button v-if="editable" type="button" tabindex="-1" aria-label="Resize camera frame" class="absolute -bottom-2 -right-2 w-5 h-5 bg-blue-500 border-2 border-white cursor-se-resize touch-none" @pointerdown.stop="drag($event, true)"></button>
  </div>
</template>
