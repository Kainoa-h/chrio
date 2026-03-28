<script setup lang="ts">
import { watch } from 'vue';

const props = defineProps<{
  message: string;
  type: 'success' | 'error' | 'info' | 'warn';
  show: boolean;
}>();

const emit = defineEmits<{ close: [] }>();

watch(() => props.show, (val) => {
  if (val) {
    setTimeout(() => emit('close'), 3000);
  }
});
</script>

<template>
  <Transition name="toast">
    <div
      v-if="show"
      class="fixed top-4 right-4 z-50 px-4 py-3 rounded-lg shadow-lg text-white text-sm font-medium flex items-center gap-2 max-w-xs"
      :class="{
        'bg-green-600': type === 'success',
        'bg-red-600': type === 'error',
        'bg-blue-600': type === 'info',
        'bg-orange-400': type === 'warn',
      }"
    >
      <span>{{ message }}</span>
      <button @click="emit('close')" class="ml-auto opacity-75 hover:opacity-100">✕</button>
    </div>
  </Transition>
</template>

<style scoped>
.toast-enter-active,
.toast-leave-active {
  transition: opacity 0.3s, transform 0.3s;
}
.toast-enter-from,
.toast-leave-to {
  opacity: 0;
  transform: translateX(1rem);
}
</style>
