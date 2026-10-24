<script setup lang="ts" generic="T extends string | number | null">
import { computed } from 'vue';
import { Check, ChevronDown, ChevronUp } from 'lucide-vue-next';
import {
  SelectContent, SelectIcon, SelectItem, SelectItemIndicator, SelectItemText,
  SelectPortal, SelectRoot, SelectScrollDownButton, SelectScrollUpButton,
  SelectTrigger, SelectValue, SelectViewport,
} from 'reka-ui';
import { cn } from '@/lib/utils';

defineOptions({ inheritAttrs: false });
const props = withDefaults(defineProps<{
  modelValue: T;
  options: { value: T; label: string; disabled?: boolean }[];
  disabled?: boolean;
  size?: 'sm' | 'default';
  placeholder?: string;
}>(), { size: 'default', placeholder: 'Select an option' });
const emit = defineEmits<{ 'update:modelValue': [value: T] }>();
// Nonempty, stable keys preserve null, empty strings, and numeric values exactly.
const optionKey = (value: T) => JSON.stringify([typeof value, value]);
const selectedKey = computed(() => optionKey(props.modelValue));
const selectedOption = computed(() => props.options.find(option => option.value === props.modelValue));
function select(key: unknown) {
  const option = props.options.find(option => optionKey(option.value) === key);
  if (option && !option.disabled) emit('update:modelValue', option.value);
}
</script>

<template>
  <SelectRoot :model-value="selectedKey" :disabled="disabled" @update:model-value="select">
    <SelectTrigger
      v-bind="$attrs"
      :class="cn('inline-flex min-w-0 items-center justify-between gap-2 rounded-lg border border-gray-300 bg-white text-left font-medium text-gray-700 shadow-sm outline-none transition-colors hover:border-gray-400 hover:bg-gray-50 focus-visible:border-blue-500 focus-visible:ring-2 focus-visible:ring-blue-500/25 disabled:cursor-not-allowed disabled:bg-gray-100 disabled:text-gray-400 disabled:opacity-70 data-[state=open]:border-blue-500 data-[state=open]:ring-2 data-[state=open]:ring-blue-500/25', size === 'sm' ? 'h-8 px-2.5 text-xs' : 'h-10 px-3 text-sm', $attrs.class as string)"
    >
      <span class="min-w-0 truncate"><SelectValue :placeholder="placeholder">{{ selectedOption?.label ?? placeholder }}</SelectValue></span>
      <SelectIcon><ChevronDown class="h-4 w-4 shrink-0 text-gray-400" /></SelectIcon>
    </SelectTrigger>
    <SelectPortal>
      <SelectContent position="popper" align="start" :side-offset="6" :collision-padding="8"
        class="z-[100] max-h-[min(20rem,var(--reka-select-content-available-height))] min-w-[var(--reka-select-trigger-width)] max-w-[calc(100vw-1rem)] overflow-hidden rounded-xl border border-gray-200 bg-white p-1 text-sm text-gray-700 shadow-xl outline-none"
      >
        <SelectScrollUpButton class="flex h-6 items-center justify-center bg-white text-gray-500"><ChevronUp class="h-4 w-4" /></SelectScrollUpButton>
        <SelectViewport class="p-0.5">
          <SelectItem v-for="option in options" :key="optionKey(option.value)" :value="optionKey(option.value)" :disabled="option.disabled" :text-value="option.label"
            class="relative flex cursor-default select-none items-center rounded-md py-2 pl-3 pr-9 outline-none data-[highlighted]:bg-blue-50 data-[highlighted]:text-blue-700 data-[state=checked]:font-semibold data-[disabled]:pointer-events-none data-[disabled]:opacity-40"
          >
            <SelectItemText>{{ option.label }}</SelectItemText>
            <SelectItemIndicator class="absolute right-2.5 flex items-center text-blue-600"><Check class="h-4 w-4" /></SelectItemIndicator>
          </SelectItem>
        </SelectViewport>
        <SelectScrollDownButton class="flex h-6 items-center justify-center bg-white text-gray-500"><ChevronDown class="h-4 w-4" /></SelectScrollDownButton>
      </SelectContent>
    </SelectPortal>
  </SelectRoot>
</template>
