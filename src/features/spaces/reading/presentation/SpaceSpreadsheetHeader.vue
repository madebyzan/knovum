<script setup lang="ts">
import type { SpreadsheetGridState } from "./use-spreadsheet-grid";

const props = defineProps<{
  columns: readonly string[];
  grid: SpreadsheetGridState;
}>();

function headerColStyle(col: string) {
  const w = props.grid.columnWidths.value[col];
  return w ? { width: `${w}px`, minWidth: `${w}px` } : { minWidth: "7.5rem" };
}
</script>

<template>
  <thead class="sticky top-0 z-20 border-b border-stroke-divider bg-surface-panel text-fg-tertiary">
    <tr>
      <th
        class="sticky left-0 z-30 w-11 cursor-pointer border-r border-stroke-divider bg-surface-panel px-2 py-1.5 text-center font-mono text-caption-sm font-medium hover:bg-surface-control-hover"
        @click="grid.selectAll()"
      >
        #
      </th>
      <th
        v-for="(col, colIdx) in columns"
        :key="col"
        class="relative cursor-pointer border-r border-stroke-divider px-3 py-1.5 font-mono text-caption-sm font-semibold last:border-r-0"
        :class="
          grid.bounds.value && colIdx >= grid.bounds.value.c0 && colIdx <= grid.bounds.value.c1
            ? 'bg-surface-accent-subtle/60 text-fg-accent'
            : 'hover:bg-surface-control-hover'
        "
        :style="headerColStyle(col)"
        @click="grid.selectColumn(colIdx, $event.shiftKey)"
      >
        <span>{{ col }}</span>
        <span
          class="absolute inset-y-0 -right-1 z-10 w-2 cursor-col-resize hover:bg-surface-accent/50"
          @mousedown="grid.startColumnResize(col, $event)"
          @dblclick.stop="grid.resetColumnWidth(col)"
        />
      </th>
    </tr>
  </thead>
</template>
