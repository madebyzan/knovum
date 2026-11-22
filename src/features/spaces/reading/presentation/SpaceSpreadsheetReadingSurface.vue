<script setup lang="ts">
import { computed, ref, shallowRef, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiFindBar from "../../../../shared/ui/UiFindBar.vue";
import type { KnowledgeLocator } from "../../knowledge/model/schema/chunk";
import type { KnowledgeChunk } from "../../knowledge/model/schema/persisted-chunk";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { projectSpreadsheetSheets } from "../spreadsheet-sheet-projection";
import SpaceSpreadsheetFormulaBar from "./SpaceSpreadsheetFormulaBar.vue";
import SpaceSpreadsheetGrid from "./SpaceSpreadsheetGrid.vue";
import SpaceSpreadsheetStatusBar from "./SpaceSpreadsheetStatusBar.vue";
import type { SurfaceSelectionState } from "./use-reader-selection";
import { useSpreadsheetGrid } from "./use-spreadsheet-grid";

const props = defineProps<{
  node: SpaceCanvasNode;
  chunks: readonly KnowledgeChunk[];
  loading: boolean;
  locator?: KnowledgeLocator | undefined;
  quote?: string | undefined;
}>();

const emit = defineEmits<{ surfaceSelection: [state: SurfaceSelectionState | undefined] }>();
const { t } = useI18n();
const gridView = useTemplateRef<InstanceType<typeof SpaceSpreadsheetGrid>>("gridView");
const selectedSheetName = shallowRef<string>();
const findOpen = shallowRef(false);
const findQuery = ref("");
const copied = shallowRef(false);

const sheets = computed(() => projectSpreadsheetSheets(props.chunks, props.node.data));
const activeSheet = computed(
  () => sheets.value.find((s) => s.name === selectedSheetName.value) ?? sheets.value[0],
);
const columns = computed(() => activeSheet.value?.columns ?? []);
const visibleRows = computed(() => {
  const sheet = activeSheet.value;
  if (!sheet) return [];
  const needle = findQuery.value.trim().toLowerCase();
  if (!needle) return sheet.rows;
  return sheet.rows.filter((row) =>
    [...row.cells.values()].some(
      (cell) => cell.value.toLowerCase().includes(needle) || cell.formula?.toLowerCase().includes(needle),
    ),
  );
});

const grid = useSpreadsheetGrid(columns, visibleRows);

const formulaText = computed(() => {
  const anchor = grid.selection.value?.anchor;
  const fallback = activeSheet.value?.name ?? "";
  if (!anchor) return fallback;
  const col = columns.value[anchor.col];
  const cell = col ? visibleRows.value[anchor.row]?.cells.get(col) : undefined;
  if (!cell) return fallback;
  return cell.formula ? `=${cell.formula}` : cell.value || fallback;
});

const highlightedChunkIndex = computed(() => {
  const loc = props.locator?.location;
  if (loc?.kind === "sheet") {
    const byRange = props.chunks.findIndex(
      (c) =>
        c.locator.location.kind === "sheet" &&
        c.locator.location.sheet === loc.sheet &&
        c.locator.location.range === loc.range,
    );
    if (byRange >= 0) return byRange;
  }
  const needle = props.quote?.trim();
  return needle ? props.chunks.findIndex((c) => c.text.includes(needle)) : -1;
});

async function copySelection() {
  const tsv = grid.getTsv();
  if (!tsv) return;
  await navigator.clipboard.writeText(tsv);
  copied.value = true;
  globalThis.setTimeout(() => {
    copied.value = false;
  }, 1400);
}

function toggleFind() {
  findOpen.value = !findOpen.value;
  if (!findOpen.value) findQuery.value = "";
}

function onKeyDown(event: KeyboardEvent) {
  if (event.key === "Escape" && grid.selection.value) {
    grid.selection.value = undefined;
    return;
  }
  if (grid.handleNavKey(event) || !(event.metaKey || event.ctrlKey)) return;
  const key = event.key.toLowerCase();
  if (key === "f") {
    event.preventDefault();
    findOpen.value = true;
  } else if (key === "c" && grid.selection.value) {
    event.preventDefault();
    void copySelection();
  } else if (key === "a") {
    event.preventDefault();
    grid.selectAll();
  }
}

watch(
  () => [props.locator, props.quote, sheets.value.length] as const,
  ([locator]) => {
    if (locator?.location.kind === "sheet") selectedSheetName.value = locator.location.sheet;
    const rowIdx = visibleRows.value.findIndex((r) => r.chunkIndex === highlightedChunkIndex.value);
    if (rowIdx >= 0) gridView.value?.scrollToRow(rowIdx);
  },
  { immediate: true },
);
watch([() => props.node.id, selectedSheetName], () => {
  grid.selection.value = undefined;
  emit("surfaceSelection", undefined);
});
</script>

<template>
  <div
    data-reader-ignore-selection
    class="flex h-full min-h-0 flex-col bg-surface-canvas outline-none select-none"
    tabindex="0"
    @keydown="onKeyDown"
  >
    <SpaceSpreadsheetFormulaBar
      v-if="activeSheet"
      :address="grid.address.value"
      :formula-text="formulaText"
      :has-selection="Boolean(grid.selection.value)"
      :copied="copied"
      :find-open="findOpen"
      @copy="copySelection"
      @toggle-find="toggleFind"
    />
    <UiFindBar
      v-if="findOpen"
      v-model="findQuery"
      :match-current="visibleRows.length ? 1 : 0"
      :match-total="visibleRows.length"
      @close="toggleFind"
    />
    <SpaceSpreadsheetGrid
      v-if="activeSheet"
      ref="gridView"
      :sheet="activeSheet"
      :columns="columns"
      :rows="visibleRows"
      :grid="grid"
      :highlighted-chunk-index="highlightedChunkIndex"
      :has-filter="Boolean(findQuery.trim())"
      @surface-selection="emit('surfaceSelection', $event)"
    />
    <SpaceSpreadsheetStatusBar
      v-if="sheets.length"
      :sheet-names="sheets.map((s) => s.name)"
      :active-sheet-name="activeSheet?.name"
      :row-count="visibleRows.length"
      :column-count="columns.length"
      :summary="grid.summary.value"
      @select-sheet="selectedSheetName = $event"
    />
    <p v-if="!activeSheet && !loading" class="p-8 text-center text-caption text-fg-tertiary">
      {{ t("spaces.conversation.sourcePeek.noText") }}
    </p>
  </div>
</template>
