import { useEventListener } from "@vueuse/core";
import { computed, ref, shallowRef, type ComputedRef } from "vue";

import {
  formatSelectionTsv,
  normalizeSelection,
  selectionAddress,
  summarizeSelection,
  type CellCoord,
  type SpreadsheetSelection,
} from "../spreadsheet-selection";
import type { SpreadsheetSheetView } from "../spreadsheet-sheet-projection";

type SheetRow = SpreadsheetSheetView["rows"][number];

/** Owns Excel/WPS-grade range selection, keyboard navigation, TSV copy, and column resizing. */
export function useSpreadsheetGrid(
  columns: ComputedRef<readonly string[]>,
  rows: ComputedRef<readonly SheetRow[]>,
) {
  const selection = shallowRef<SpreadsheetSelection>();
  const dragging = shallowRef(false);
  const columnWidths = ref<Record<string, number>>({});
  const bounds = computed(() => (selection.value ? normalizeSelection(selection.value) : undefined));
  const address = computed(() => selectionAddress(columns.value, rows.value, selection.value));
  const summary = computed(() => summarizeSelection(columns.value, rows.value, selection.value));

  function clamp(row: number, col: number): CellCoord {
    return {
      row: Math.min(Math.max(0, row), Math.max(0, rows.value.length - 1)),
      col: Math.min(Math.max(0, col), Math.max(0, columns.value.length - 1)),
    };
  }

  function startCellDrag(row: number, col: number, shiftKey: boolean) {
    const coord = clamp(row, col);
    if (shiftKey && selection.value) {
      selection.value = { anchor: selection.value.anchor, focus: coord };
      return;
    }
    selection.value = { anchor: coord, focus: coord };
    dragging.value = true;
  }

  function extendCellDrag(row: number, col: number) {
    if (!dragging.value || !selection.value) return;
    const next = clamp(row, col);
    if (selection.value.focus.row === next.row && selection.value.focus.col === next.col) return;
    selection.value = { anchor: selection.value.anchor, focus: next };
  }

  function selectColumn(colIdx: number, shiftKey: boolean) {
    if (!rows.value.length) return;
    const endRow = rows.value.length - 1;
    if (shiftKey && selection.value) {
      selection.value = {
        anchor: { row: 0, col: selection.value.anchor.col },
        focus: { row: endRow, col: colIdx },
      };
      return;
    }
    selection.value = { anchor: { row: 0, col: colIdx }, focus: { row: endRow, col: colIdx } };
  }

  function selectRow(rowIdx: number, shiftKey: boolean) {
    if (!columns.value.length) return;
    const endCol = columns.value.length - 1;
    if (shiftKey && selection.value) {
      selection.value = {
        anchor: { row: selection.value.anchor.row, col: 0 },
        focus: { row: rowIdx, col: endCol },
      };
      return;
    }
    selection.value = { anchor: { row: rowIdx, col: 0 }, focus: { row: rowIdx, col: endCol } };
  }

  function selectAll() {
    if (!rows.value.length || !columns.value.length) return;
    selection.value = {
      anchor: { row: 0, col: 0 },
      focus: { row: rows.value.length - 1, col: columns.value.length - 1 },
    };
  }

  function moveBy(dRow: number, dCol: number, extend: boolean) {
    if (!rows.value.length || !columns.value.length) return;
    const current = selection.value;
    if (!current) {
      selection.value = { anchor: { row: 0, col: 0 }, focus: { row: 0, col: 0 } };
      return;
    }
    const base = extend ? current.focus : current.anchor;
    const next = clamp(base.row + dRow, base.col + dCol);
    selection.value = extend ? { anchor: current.anchor, focus: next } : { anchor: next, focus: next };
  }

  function getTsv() {
    return formatSelectionTsv(columns.value, rows.value, selection.value);
  }

  function startColumnResize(col: string, event: MouseEvent) {
    event.preventDefault();
    event.stopPropagation();
    const header = (event.currentTarget as HTMLElement | null)?.parentElement;
    const startWidth = columnWidths.value[col] ?? header?.offsetWidth ?? 140;
    const startX = event.clientX;
    const onMove = (moveEvent: MouseEvent) => {
      const nextWidth = Math.min(640, Math.max(72, Math.round(startWidth + moveEvent.clientX - startX)));
      columnWidths.value = { ...columnWidths.value, [col]: nextWidth };
    };
    const onUp = () => {
      globalThis.removeEventListener("mousemove", onMove);
      globalThis.removeEventListener("mouseup", onUp);
    };
    globalThis.addEventListener("mousemove", onMove);
    globalThis.addEventListener("mouseup", onUp);
  }

  function resetColumnWidth(col: string) {
    const next = { ...columnWidths.value };
    delete next[col];
    columnWidths.value = next;
  }

  function handleNavKey(event: KeyboardEvent): boolean {
    const key = event.key;
    if (key === "ArrowUp") moveBy(-1, 0, event.shiftKey);
    else if (key === "ArrowDown") moveBy(1, 0, event.shiftKey);
    else if (key === "ArrowLeft") moveBy(0, -1, event.shiftKey);
    else if (key === "ArrowRight") moveBy(0, 1, event.shiftKey);
    else if (key === "Enter") moveBy(event.shiftKey ? -1 : 1, 0, false);
    else if (key === "Tab") moveBy(0, event.shiftKey ? -1 : 1, false);
    else return false;
    event.preventDefault();
    return true;
  }

  useEventListener(globalThis, "mouseup", () => {
    dragging.value = false;
  });

  return {
    selection,
    dragging,
    bounds,
    address,
    summary,
    columnWidths,
    startCellDrag,
    extendCellDrag,
    selectColumn,
    selectRow,
    selectAll,
    handleNavKey,
    getTsv,
    startColumnResize,
    resetColumnWidth,
  };
}

export type SpreadsheetGridState = ReturnType<typeof useSpreadsheetGrid>;
