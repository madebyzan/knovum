import { classifyFilePresentation } from "../assets/file-presentation";
import type { KnowledgeChunk } from "../knowledge/model/schema/persisted-chunk";
import type { SpaceNodePayload } from "../model/nodes/node-payload";

interface SpreadsheetCell {
  column: string;
  value: string;
  formula?: string | undefined;
}

interface SpreadsheetRow {
  rowLabel: string;
  chunkIndex: number;
  cells: ReadonlyMap<string, SpreadsheetCell>;
}

export interface SpreadsheetSheetView {
  name: string;
  columns: readonly string[];
  rows: readonly SpreadsheetRow[];
}

const SHEET_HEADER_PREFIX = /^First row context \(not a declared header\):\n[^\n]*\n\n/u;
const CELL_TOKEN_ANCHOR = /(?:^|[\t\r\n])(?:([A-Z]+)(\d+)|R(\d+)C(\d+)):[ \t]?/gu;
const FORMULA_PATTERN = /^formula=(.+?);\s*cached=(.*)$/su;

export function isSpreadsheetPayload(payload: SpaceNodePayload): boolean {
  if (payload.kind !== "file") return false;
  const presentation = classifyFilePresentation(payload.fileName);
  return presentation === "spreadsheet" || presentation === "tabular-data";
}

function columnLetters(index: number): string {
  let current = index;
  let label = "";
  while (current > 0) {
    const rem = (current - 1) % 26;
    label = String.fromCharCode(65 + rem) + label;
    current = Math.floor((current - 1) / 26);
  }
  return label || "A";
}

function parseCellPayload(rawValue: string): { value: string; formula?: string | undefined } {
  const trimmed = rawValue.trim();
  const formulaMatch = FORMULA_PATTERN.exec(trimmed);
  if (!formulaMatch) return { value: trimmed };
  return { value: (formulaMatch[2] ?? "").trim(), formula: formulaMatch[1]!.trim() };
}

function sortColumns(columns: Iterable<string>): string[] {
  return [...columns].toSorted((left, right) =>
    left.length !== right.length ? left.length - right.length : left.localeCompare(right),
  );
}

interface SheetAccumulator {
  columns: Set<string>;
  rowsByLabel: Map<string, { chunkIndex: number; cells: Map<string, SpreadsheetCell> }>;
  rowOrder: string[];
}

function upsertCell(sheet: SheetAccumulator, rowLabel: string, cell: SpreadsheetCell, chunkIndex: number) {
  let row = sheet.rowsByLabel.get(rowLabel);
  if (!row) {
    row = { chunkIndex, cells: new Map<string, SpreadsheetCell>() };
    sheet.rowsByLabel.set(rowLabel, row);
    sheet.rowOrder.push(rowLabel);
  }
  sheet.columns.add(cell.column);
  row.cells.set(cell.column, cell);
}

function appendStructuredCells(sheet: SheetAccumulator, cleaned: string, chunkIndex: number): boolean {
  const matches = [...cleaned.matchAll(CELL_TOKEN_ANCHOR)];
  if (!matches.length) return false;
  for (let i = 0; i < matches.length; i += 1) {
    const match = matches[i]!;
    const valueStart = (match.index ?? 0) + match[0].length;
    const valueEnd = i + 1 < matches.length ? (matches[i + 1]!.index ?? cleaned.length) : cleaned.length;
    const column = match[1] ?? columnLetters(Number(match[4]));
    const rowLabel = match[2] ?? match[3]!;
    upsertCell(
      sheet,
      rowLabel,
      { column, ...parseCellPayload(cleaned.slice(valueStart, valueEnd)) },
      chunkIndex,
    );
  }
  return true;
}

function appendDelimitedRows(sheet: SheetAccumulator, cleaned: string, chunkIndex: number) {
  for (const line of cleaned.split(/\r?\n/u)) {
    const parts = line.split("\t").map((part) => part.trim());
    if (!parts.some(Boolean)) continue;
    const rowLabel = String(sheet.rowOrder.length + 1);
    for (const [colIdx, part] of parts.entries()) {
      if (!part) continue;
      upsertCell(
        sheet,
        rowLabel,
        { column: columnLetters(colIdx + 1), ...parseCellPayload(part) },
        chunkIndex,
      );
    }
  }
}

function appendTextToSheet(sheet: SheetAccumulator, text: string, chunkIndex: number) {
  const cleaned = text.replace(SHEET_HEADER_PREFIX, "").trim();
  if (!cleaned) return;
  if (!appendStructuredCells(sheet, cleaned, chunkIndex)) {
    appendDelimitedRows(sheet, cleaned, chunkIndex);
  }
}

function ensureSheet(bySheet: Map<string, SheetAccumulator>, name: string): SheetAccumulator {
  let entry = bySheet.get(name);
  if (!entry) {
    entry = { columns: new Set<string>(), rowsByLabel: new Map(), rowOrder: [] };
    bySheet.set(name, entry);
  }
  return entry;
}

function isReadableSheetChunk(chunk: KnowledgeChunk): boolean {
  return chunk.locator.location.kind === "sheet" && chunk.kind !== "node";
}

export function projectSpreadsheetSheets(
  chunks: readonly KnowledgeChunk[],
  payload: SpaceNodePayload,
): readonly SpreadsheetSheetView[] {
  const bySheet = new Map<string, SheetAccumulator>();
  const sheetEntries = chunks
    .map((chunk, chunkIndex) => ({ chunk, chunkIndex }))
    .filter(({ chunk }) => isReadableSheetChunk(chunk));

  if (sheetEntries.length) {
    for (const { chunk, chunkIndex } of sheetEntries) {
      if (chunk.locator.location.kind !== "sheet") continue;
      appendTextToSheet(ensureSheet(bySheet, chunk.locator.location.sheet), chunk.text, chunkIndex);
    }
  } else if (payload.kind === "file") {
    const raw = payload.extractedText?.trim() || payload.previewText?.trim();
    if (raw) {
      const normalized = payload.fileName.toLowerCase().endsWith(".csv") ? raw.replace(/,/gu, "\t") : raw;
      appendTextToSheet(ensureSheet(bySheet, payload.title), normalized, 0);
    }
  }

  return [...bySheet.entries()]
    .filter(([, value]) => value.rowOrder.length > 0)
    .map(([name, value]) => ({
      name,
      columns: sortColumns(value.columns),
      rows: value.rowOrder.map((rowLabel) => {
        const entry = value.rowsByLabel.get(rowLabel)!;
        return { rowLabel, chunkIndex: entry.chunkIndex, cells: entry.cells };
      }),
    }));
}
