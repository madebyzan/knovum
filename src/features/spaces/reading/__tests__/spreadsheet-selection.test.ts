import { describe, expect, it } from "vitest";

import {
  formatSelectionTsv,
  isCellInRange,
  normalizeSelection,
  selectionAddress,
  summarizeSelection,
} from "../spreadsheet-selection";

const COLUMNS = ["A", "B", "C"] as const;
const ROWS = [
  {
    rowLabel: "1",
    chunkIndex: 0,
    cells: new Map([
      ["A", { column: "A", value: "Item" }],
      ["B", { column: "B", value: "Qty" }],
      ["C", { column: "C", value: "Note" }],
    ]),
  },
  {
    rowLabel: "2",
    chunkIndex: 0,
    cells: new Map([
      ["A", { column: "A", value: "Latte" }],
      ["B", { column: "B", value: "1,200" }],
      ["C", { column: "C", value: "Line 1\nLine 2" }],
    ]),
  },
  {
    rowLabel: "3",
    chunkIndex: 0,
    cells: new Map([
      ["A", { column: "A", value: "Mocha" }],
      ["B", { column: "B", value: "800" }],
      ["C", { column: "C", value: "Hot" }],
    ]),
  },
];

describe("spreadsheet-selection", () => {
  it("normalizes reverse drag ranges and computes Excel range addresses", () => {
    const sel = { anchor: { row: 2, col: 2 }, focus: { row: 1, col: 0 } };
    const bounds = normalizeSelection(sel);
    expect(bounds).toEqual({ r0: 1, r1: 2, c0: 0, c1: 2 });
    expect(selectionAddress(COLUMNS, ROWS, sel)).toBe("A2:C3");
    expect(isCellInRange(1, 1, bounds)).toBe(true);
    expect(isCellInRange(0, 1, bounds)).toBe(false);
  });

  it("formats multi-cell selections as Excel-safe TSV and computes numeric summaries", () => {
    const sel = { anchor: { row: 1, col: 0 }, focus: { row: 2, col: 2 } };
    expect(formatSelectionTsv(COLUMNS, ROWS, sel)).toBe('Latte\t1,200\t"Line 1\nLine 2"\nMocha\t800\tHot');
    expect(summarizeSelection(COLUMNS, ROWS, sel)).toEqual({
      cellCount: 6,
      nonEmptyCount: 6,
      numericCount: 2,
      sum: 2000,
      average: 1000,
    });
  });
});
