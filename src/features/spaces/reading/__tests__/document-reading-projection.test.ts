import { describe, expect, it } from "vitest";

import type { KnowledgeChunk } from "../../knowledge/model/schema/persisted-chunk";
import type { SpaceNodePayload } from "../../model/nodes/node-payload";
import { projectDocumentBlocks } from "../document-block-projection";
import { isSpreadsheetPayload, projectSpreadsheetSheets } from "../spreadsheet-sheet-projection";

function makeChunk(
  id: string,
  kind: KnowledgeChunk["kind"],
  text: string,
  location: KnowledgeChunk["locator"]["location"],
): KnowledgeChunk {
  return {
    id,
    spaceId: "space",
    nodeId: "doc",
    origin: "source",
    sourceRevision: "sha256:rev1",
    kind,
    text,
    createdAt: 1,
    updatedAt: 1,
    locator: { sourceRevision: "sha256:rev1", location },
  };
}

describe("spreadsheet-sheet-projection", () => {
  it("parses Excel A1 cells, formulas, and multiple sheets into structured tables", () => {
    const payload: SpaceNodePayload = {
      kind: "file",
      title: "Financials",
      fileName: "financials.xlsx",
      assetPath: "/assets/financials.xlsx",
      sourceRevision: "sha256:rev1",
    };
    const chunks = [
      makeChunk("n0", "node", "financials.xlsx financials.xlsx", { kind: "node" }),
      makeChunk(
        "c1",
        "table",
        "A1: Item\tB1: Revenue\nA2: Espresso\n1. Dark roast\n2. Single origin\tB2: formula=SUM(C2:D2); cached=1250",
        {
          kind: "sheet",
          sheet: "Summary",
          range: "1:2",
        },
      ),
      makeChunk(
        "c2",
        "table",
        "First row context (not a declared header):\nA1: Item\tB1: Revenue\n\nA3: Latte\tB3: 980",
        { kind: "sheet", sheet: "Summary", range: "3:3" },
      ),
      makeChunk("c3", "table", "R1C1: Metric\tR1C2: Value\nR2C1: Margin\tR2C2: 68%", {
        kind: "sheet",
        sheet: "KPIs",
        range: "1:2",
      }),
    ];

    expect(isSpreadsheetPayload(payload)).toBe(true);
    const sheets = projectSpreadsheetSheets(chunks, payload);
    expect(sheets.map((sheet) => sheet.name)).toEqual(["Summary", "KPIs"]);
    expect(sheets[0]!.columns).toEqual(["A", "B"]);
    expect(sheets[0]!.rows).toHaveLength(3);
    expect(sheets[0]!.rows[1]!.cells.get("A")?.value).toBe("Espresso\n1. Dark roast\n2. Single origin");
    expect(sheets[0]!.rows[1]!.cells.get("B")).toEqual({
      column: "B",
      value: "1250",
      formula: "SUM(C2:D2)",
    });
    expect(sheets[0]!.rows[2]!.rowLabel).toBe("3");
    expect(sheets[1]!.rows[1]!.cells.get("B")?.value).toBe("68%");
  });

  it("falls back to CSV text when spreadsheet chunks are not yet indexed", () => {
    const payload: SpaceNodePayload = {
      kind: "file",
      title: "Metrics",
      fileName: "metrics.csv",
      assetPath: "/assets/metrics.csv",
      previewText: "City,Orders\nHarbor,190",
    };
    const sheets = projectSpreadsheetSheets([], payload);
    expect(sheets).toHaveLength(1);
    expect(sheets[0]!.rows[1]!.cells.get("A")?.value).toBe("Harbor");
    expect(sheets[0]!.rows[1]!.cells.get("B")?.value).toBe("190");
  });
});

describe("document-block-projection", () => {
  it("projects Word headings, tables, Slides, and numbered Code lines cleanly", () => {
    const docPayload: SpaceNodePayload = {
      kind: "file",
      title: "Proposal",
      fileName: "proposal.docx",
      assetPath: "/assets/proposal.docx",
    };
    const chunks = [
      makeChunk("n0", "node", "proposal.docx proposal.docx", { kind: "node" }),
      makeChunk("h1", "heading", "1. Executive Summary", {
        kind: "word",
        section: 0,
        element: 0,
        headingPath: ["1. Executive Summary"],
      }),
      makeChunk("t1", "table", "Table header:\nItem\tCost\n\nFit-out\t86000\nRent\t14500", {
        kind: "word",
        section: 0,
        element: 1,
        headingPath: ["1. Executive Summary"],
      }),
      makeChunk("s1", "slide", "Key Findings on Slide 2", { kind: "slides", slide: 2, element: 0 }),
      makeChunk("cd1", "code", 'fn main() {\n  println!("ok");\n}', {
        kind: "code",
        lineStart: 10,
        lineEnd: 12,
      }),
    ];

    const blocks = projectDocumentBlocks(chunks, docPayload);
    expect(blocks[0]).toMatchObject({ kind: "heading", text: "1. Executive Summary" });
    expect(blocks[1]).toMatchObject({
      kind: "table",
      tableRows: [
        ["Fit-out", "86000"],
        ["Rent", "14500"],
      ],
    });
    expect(blocks[2]).toMatchObject({ kind: "slide", slideNumber: 2 });
    expect(blocks[3]).toMatchObject({
      kind: "code",
      codeLines: [
        { number: 10, text: "fn main() {" },
        { number: 11, text: '  println!("ok");' },
        { number: 12, text: "}" },
      ],
    });
  });
});
