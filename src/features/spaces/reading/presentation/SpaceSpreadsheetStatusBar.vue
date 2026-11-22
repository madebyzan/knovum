<script setup lang="ts">
import { Table2 } from "@lucide/vue";

import type { SelectionSummary } from "../spreadsheet-selection";

defineProps<{
  sheetNames: readonly string[];
  activeSheetName?: string | undefined;
  rowCount: number;
  columnCount: number;
  summary: SelectionSummary;
}>();

const emit = defineEmits<{ selectSheet: [name: string] }>();
</script>

<template>
  <div
    class="flex h-8 shrink-0 items-center justify-between gap-2 border-t border-stroke-divider bg-surface-panel px-2.5"
  >
    <div class="flex min-w-0 items-center gap-1 overflow-x-auto">
      <button
        v-for="name in sheetNames"
        :key="name"
        type="button"
        class="inline-flex shrink-0 items-center gap-1.5 rounded-sm border px-2.5 py-0.5 text-caption-sm font-medium transition-colors"
        :class="
          activeSheetName === name
            ? 'border-stroke-divider bg-surface-raised text-fg-primary shadow-2xs'
            : 'border-transparent text-fg-secondary hover:bg-surface-control-hover hover:text-fg-primary'
        "
        @click="emit('selectSheet', name)"
      >
        <Table2 class="size-3 text-fg-tertiary" />
        <span class="max-w-44 truncate">{{ name }}</span>
      </button>
    </div>
    <div class="flex shrink-0 items-center gap-3 font-mono text-[11px] text-fg-tertiary tabular-nums">
      <template v-if="summary.cellCount > 1">
        <span>Count {{ summary.nonEmptyCount }}</span>
        <span v-if="summary.numericCount > 0">
          Sum {{ Number(summary.sum.toFixed(2)).toLocaleString() }}
        </span>
        <span v-if="summary.numericCount > 1">
          Avg {{ Number(summary.average.toFixed(2)).toLocaleString() }}
        </span>
      </template>
      <span v-else-if="activeSheetName">{{ rowCount }} × {{ columnCount }}</span>
    </div>
  </div>
</template>
