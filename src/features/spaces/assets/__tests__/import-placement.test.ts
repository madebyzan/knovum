import { describe, expect, it } from "vitest";

import { layoutImportGrid } from "../import-placement";

describe("Space import placement", () => {
  it("uses the widest column and tallest row so mixed resources never overlap", () => {
    expect(
      layoutImportGrid(
        [
          { width: 420, height: 300 },
          { width: 300, height: 520 },
          { width: 360, height: 96 },
          { width: 280, height: 220 },
        ],
        { x: 100, y: 80 },
      ),
    ).toEqual([
      { x: 100, y: 80 },
      { x: 544, y: 80 },
      { x: 100, y: 624 },
      { x: 544, y: 624 },
    ]);
  });
});
