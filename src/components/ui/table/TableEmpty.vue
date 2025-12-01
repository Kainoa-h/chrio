<script setup lang="ts">
import { type HTMLAttributes } from 'vue'
import { cn } from '@/lib/utils'
import TableRow from './TableRow.vue'
import TableCell from './TableCell.vue'

const props = withDefaults(defineProps<{
  class?: HTMLAttributes['class']
  colspan?: number
}>(), {
  colspan: 1
})
</script>

<template>
  <TableRow>
    <TableCell
      :class="cn('p-4 whitespace-nowrap align-middle text-sm text-foreground', props.class)"
      :colspan="props.colspan"
    >
      <div class="flex items-center justify-center py-10">
        <slot />
      </div>
    </TableCell>
  </TableRow>
</template>
