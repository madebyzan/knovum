<script setup lang="ts">
import { useVirtualizer } from "@tanstack/vue-virtual";
import { useEventListener } from "@vueuse/core";
import { computed, nextTick, onBeforeUnmount, useTemplateRef, watch } from "vue";

import { spreadsheetCellStyle } from "../spreadsheet-selection";
import type { SpreadsheetSheetView } from "../spreadsheet-sheet-projection";
import SpaceSpreadsheetHeader from "./SpaceSpreadsheetHeader.vue";
import { anchorElementRange, type SurfaceSelectionState } from "./use-reader-selection";
import type { SpreadsheetGridState } from "./use-spreadsheet-grid";

const props = defineProps<{
  sheet: SpreadsheetSheetView;
  columns: readonly string[];
  rows: SpreadsheetSheetView["rows"];
  grid: SpreadsheetGridState;
  highlightedChunkIndex: number;
  hasFilter: boolean;
}>();

const emit = defineEmits<{ surfaceSelection: [state: SurfaceSelectionState | undefined] }>();
const scroller = useTemplateRef<HTMLDivElement>("scroller");
let scrollFrameId: number | undefined;
let measureEpoch = 0;

const virtualizer = useVirtualizer(
  computed(() => ({
    count: props.rows.length,
    getScrollElement: () => scroller.value,
    estimateSize: () => 36,
    overscan: 12,
  })),
);
const virtualRows = computed(() => virtualizer.value.getVirtualItems());
const paddingTop = computed(() => virtualRows.value[0]?.start ?? 0);
const paddingBottom = computed(() => {
  const last = virtualRows.value.at(-1);
  return last ? Math.max(0, virtualizer.value.getTotalSize() - last.end) : 0;
});
const selectedTsv = computed(() =>
  props.grid.bounds.value && !props.grid.dragging.value ? props.grid.getTsv().trim() : "",
);

function cellStyle(row: number, col: number, colName: string) {
  return spreadsheetCellStyle(
    { row, col },
    props.grid.selection.value,
    props.grid.bounds.value,
    props.grid.columnWidths.value[colName],
  );
}

function rowClass(index: number) {
  if (props.rows[index]!.chunkIndex === props.highlightedChunkIndex) return "bg-surface-accent-subtle/35";
  return index === 0 && !props.hasFilter ? "bg-surface-control/55 font-medium" : "even:bg-surface-control/15";
}

function cellFormula(rowIdx: number, col: string) {
  const formula = props.rows[rowIdx]!.cells.get(col)?.formula;
  return formula ? `=${formula}` : undefined;
}

function measureRow(el: unknown) {
  if (!(el instanceof HTMLElement)) return;
  const key = `${el.dataset.index ?? ""}:${measureEpoch}`;
  if (el.dataset.measured === key) return;
  el.dataset.measured = key;
  virtualizer.value.measureElement(el);
}

function syncSurfaceSelection() {
  const b = props.grid.bounds.value;
  const container = scroller.value;
  const text = selectedTsv.value;
  if (!b || !container || !text) {
    emit("surfaceSelection", undefined);
    return;
  }
  const first = container.querySelector(`td[data-cell-row="${b.r0}"][data-cell-col="${b.c0}"]`);
  const last = container.querySelector(`td[data-cell-row="${b.r1}"][data-cell-col="${b.c1}"]`);
  const anchor = anchorElementRange(first, last, container.closest("[data-reader-body]") ?? container);
  emit(
    "surfaceSelection",
    anchor
      ? {
          passage: {
            text,
            location: { kind: "sheet", sheet: props.sheet.name, range: props.grid.address.value },
          },
          anchor,
          clear: () => {
            props.grid.selection.value = undefined;
          },
        }
      : undefined,
  );
}

function scheduleScrollSync() {
  if (!selectedTsv.value || scrollFrameId !== undefined) return;
  scrollFrameId = requestAnimationFrame(() => {
    scrollFrameId = undefined;
    syncSurfaceSelection();
  });
}

defineExpose({
  scrollToRow: (index: number) => virtualizer.value.scrollToIndex(index, { align: "center" }),
});
watch([selectedTsv, () => props.sheet.name], () => void nextTick(syncSurfaceSelection));
watch(props.grid.columnWidths, () => {
  measureEpoch += 1;
  void nextTick(() => virtualizer.value.measure());
});
useEventListener(scroller, "scroll", scheduleScrollSync, { passive: true });
onBeforeUnmount(() => {
  if (scrollFrameId !== undefined) cancelAnimationFrame(scrollFrameId);
  emit("surfaceSelection", undefined);
});
</script>

<template>
  <div ref="scroller" class="min-h-0 flex-1 overflow-auto bg-surface-raised">
    <table class="w-max min-w-full border-collapse text-left text-caption">
      <SpaceSpreadsheetHeader :columns="columns" :grid="grid" />
      <tbody class="divide-y divide-stroke-divider/70">
        <tr v-if="paddingTop > 0">
          <td :colspan="columns.length + 1" :style="{ height: `${paddingTop}px`, padding: 0 }" />
        </tr>
        <tr
          v-for="vRow in virtualRows"
          :key="vRow.index"
          :ref="measureRow"
          :data-index="vRow.index"
          :data-chunk-index="rows[vRow.index]!.chunkIndex"
          :class="rowClass(vRow.index)"
        >
          <td
            class="sticky left-0 z-10 cursor-pointer border-r border-stroke-divider px-2 py-2 text-center align-top font-mono text-caption-sm tabular-nums"
            :class="
              grid.bounds.value && vRow.index >= grid.bounds.value.r0 && vRow.index <= grid.bounds.value.r1
                ? 'bg-surface-accent-subtle/70 font-semibold text-fg-accent'
                : 'bg-surface-panel/95 text-fg-tertiary hover:bg-surface-control-hover'
            "
            @click="grid.selectRow(vRow.index, $event.shiftKey)"
          >
            {{ rows[vRow.index]!.rowLabel }}
          </td>
          <td
            v-for="(col, colIdx) in columns"
            :key="col"
            :data-cell-row="vRow.index"
            :data-cell-col="colIdx"
            class="cursor-cell border-r border-stroke-divider/60 px-3 py-2 align-top text-fg-primary last:border-r-0"
            :style="cellStyle(vRow.index, colIdx, col)"
            :title="cellFormula(vRow.index, col)"
            @mousedown.prevent="grid.startCellDrag(vRow.index, colIdx, $event.shiftKey)"
            @mouseenter="grid.extendCellDrag(vRow.index, colIdx)"
          >
            <div class="flex items-baseline justify-between gap-2">
              <span class="leading-relaxed break-words whitespace-pre-wrap">{{
                rows[vRow.index]!.cells.get(col)?.value ?? ""
              }}</span>
              <span
                v-if="cellFormula(vRow.index, col)"
                class="shrink-0 rounded-xs border border-stroke-divider bg-surface-control px-1 font-mono text-[10px] text-fg-tertiary"
                >fx</span
              >
            </div>
          </td>
        </tr>
        <tr v-if="paddingBottom > 0">
          <td :colspan="columns.length + 1" :style="{ height: `${paddingBottom}px`, padding: 0 }" />
        </tr>
      </tbody>
    </table>
  </div>
</template>
