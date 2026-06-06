<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue';

defineProps<{
  show: boolean;
  title: string;
  message: string;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
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
  <div v-if="show" class="fixed inset-0 bg-black/50 z-50 flex items-center justify-center" @click="$emit('close')">
    <div class="bg-white rounded-xl shadow-xl p-6 max-w-sm w-full mx-4 space-y-4" @click.stop>
      <h2 class="text-lg font-semibold text-gray-900">{{ title }}</h2>
      <p class="text-sm text-gray-600">{{ message }}</p>
      <div class="flex flex-col gap-2">
        <slot name="actions" />
      </div>
    </div>
  </div>
</template>
