import type { SpreadsheetSheetView } from "./spreadsheet-sheet-projection";

export interface CellCoord {
  row: number;
  col: number;
}

export interface SpreadsheetSelection {
  anchor: CellCoord;
  focus: CellCoord;
}

export interface NormalizedRange {
  r0: number;
  r1: number;
  c0: number;
  c1: number;
}

export interface SelectionSummary {
  cellCount: number;
  nonEmptyCount: number;
  numericCount: number;
  sum: number;
  average: number;
}

type SheetRow = SpreadsheetSheetView["rows"][number];

const NUMERIC_VALUE = /^[+-]?(?:\d{1,3}(?:,\d{3})+|\d+)(?:\.\d+)?%?$/u;

export function normalizeSelection(selection: SpreadsheetSelection): NormalizedRange {
  return {
    r0: Math.min(selection.anchor.row, selection.focus.row),
    r1: Math.max(selection.anchor.row, selection.focus.row),
    c0: Math.min(selection.anchor.col, selection.focus.col),
    c1: Math.max(selection.anchor.col, selection.focus.col),
  };
}

function cellLabel(columns: readonly string[], rows: readonly SheetRow[], r: number, c: number): string {
  return `${columns[c] ?? "A"}${rows[r]?.rowLabel ?? String(r + 1)}`;
}

export function selectionAddress(
  columns: readonly string[],
  rows: readonly SheetRow[],
  selection: SpreadsheetSelection | undefined,
): string {
  if (!selection || !columns.length || !rows.length) return "A1";
  const { r0, r1, c0, c1 } = normalizeSelection(selection);
  const start = cellLabel(columns, rows, r0, c0);
  const end = cellLabel(columns, rows, r1, c1);
  return start === end ? start : `${start}:${end}`;
}

function escapeTsvCell(value: string): string {
  if (!/[\t\r\n"]/u.test(value)) return value;
  return `"${value.replaceAll('"', '""')}"`;
}

/** Formats a rectangular selection as Excel/WPS-compatible TSV (preserving embedded newlines). */
export function formatSelectionTsv(
  columns: readonly string[],
  rows: readonly SheetRow[],
  selection: SpreadsheetSelection | undefined,
): string {
  if (!selection) return "";
  const { r0, r1, c0, c1 } = normalizeSelection(selection);
  const lines: string[] = [];
  for (let r = r0; r <= r1; r += 1) {
    const row = rows[r];
    if (!row) continue;
    const cells: string[] = [];
    for (let c = c0; c <= c1; c += 1) {
      const col = columns[c];
      cells.push(escapeTsvCell(col ? (row.cells.get(col)?.value ?? "") : ""));
    }
    lines.push(cells.join("\t"));
  }
  return lines.join("\n");
}

function parseNumericCell(raw: string): number | undefined {
  const trimmed = raw.trim();
  if (!trimmed || !NUMERIC_VALUE.test(trimmed)) return undefined;
  const isPercent = trimmed.endsWith("%");
  const cleaned = trimmed.replaceAll(",", "").replace(/%$/u, "");
  const num = Number(cleaned);
  if (!Number.isFinite(num)) return undefined;
  return isPercent ? num / 100 : num;
}

function trimmedCellValue(columns: readonly string[], row: SheetRow | undefined, c: number): string {
  const col = columns[c];
  return col && row ? (row.cells.get(col)?.value ?? "").trim() : "";
}

/** Computes Excel/WPS-style selection statistics (count, numeric sum, average). */
export function summarizeSelection(
  columns: readonly string[],
  rows: readonly SheetRow[],
  selection: SpreadsheetSelection | undefined,
): SelectionSummary {
  if (!selection) return { cellCount: 0, nonEmptyCount: 0, numericCount: 0, sum: 0, average: 0 };
  const { r0, r1, c0, c1 } = normalizeSelection(selection);
  let cellCount = 0;
  let nonEmptyCount = 0;
  let numericCount = 0;
  let sum = 0;
  for (let r = r0; r <= r1; r += 1) {
    for (let c = c0; c <= c1; c += 1) {
      cellCount += 1;
      const val = trimmedCellValue(columns, rows[r], c);
      if (!val) continue;
      nonEmptyCount += 1;
      const numeric = parseNumericCell(val);
      if (numeric !== undefined) {
        numericCount += 1;
        sum += numeric;
      }
    }
  }
  return { cellCount, nonEmptyCount, numericCount, sum, average: numericCount ? sum / numericCount : 0 };
}

export function isCellInRange(row: number, col: number, bounds: NormalizedRange | undefined): boolean {
  return Boolean(bounds && row >= bounds.r0 && row <= bounds.r1 && col >= bounds.c0 && col <= bounds.c1);
}

function selectionEdgeShadow(row: number, col: number, bounds: NormalizedRange): string {
  const edges: string[] = [];
  if (row === bounds.r0) edges.push("inset 0 2px 0 0 var(--color-surface-accent)");
  if (row === bounds.r1) edges.push("inset 0 -2px 0 0 var(--color-surface-accent)");
  if (col === bounds.c0) edges.push("inset 2px 0 0 0 var(--color-surface-accent)");
  if (col === bounds.c1) edges.push("inset -2px 0 0 0 var(--color-surface-accent)");
  return edges.join(", ");
}

export function spreadsheetCellStyle(
  coord: CellCoord,
  selection: SpreadsheetSelection | undefined,
  bounds: NormalizedRange | undefined,
  width: number | undefined,
): Record<string, string> {
  const base: Record<string, string> = width
    ? { width: `${width}px`, maxWidth: `${width}px` }
    : { maxWidth: "24rem" };
  if (!bounds || !isCellInRange(coord.row, coord.col, bounds)) return base;
  const isAnchor = selection?.anchor.row === coord.row && selection.anchor.col === coord.col;
  return {
    ...base,
    backgroundColor: isAnchor
      ? "var(--color-surface-raised)"
      : "color-mix(in srgb, var(--color-surface-accent) 11%, transparent)",
    boxShadow: selectionEdgeShadow(coord.row, coord.col, bounds),
  };
}
