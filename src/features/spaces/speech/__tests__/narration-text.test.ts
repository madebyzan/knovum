import { expect, it } from "vitest";

import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { narrationText } from "../narration-text";

function markdown(text: string): SpaceCanvasNode {
  return {
    id: "source",
    type: "markdown",
    initialPosition: { x: 0, y: 0 },
    data: { kind: "markdown", title: "Source file", text },
  };
}

it("keeps headings, prose and line breaks without reading Markdown formatting or URLs", () => {
  const text = narrationText(
    markdown(
      "# Review\n\n**Important** [decision](https://example.com)\nNext line.\n\n- First item\n- Second item",
    ),
  );
  expect(text).toContain("Review");
  expect(text).toContain("Important decision\nNext line.");
  expect(text).toContain("First item");
  expect(text).not.toContain("https://");
  expect(text).not.toContain("Source file");
  expect(text).not.toContain("**");
});

it("reads a heading-only manuscript without losing its content", () => {
  expect(narrationText(markdown("# Hello"))).toBe("Hello");
  expect(narrationText(markdown(""))).toBe("");
});

it("strips citations, footnotes, timestamps, code blocks, tables, LaTeX, URLs, and non-speakable symbols", () => {
  const text = narrationText(
    markdown(
      [
        "# Opening [C1]",
        "",
        "Looking back on the first one [C2][C3]. Watson said it felt strange (01:10–01:11)[^1].",
        "",
        "Tickets cost $5 to $10, while $\\frac{a}{b}$ and $$\\sum_{i=1}^n x_i$$ are skipped.",
        "",
        "Visit https://example.com/path/123 🔥✨ → done\u200B!",
        "",
        "```python",
        "print('do not read code')",
        "```",
        "",
        "| Metric | Value |",
        "| --- | --- |",
        "| Margin | 68% |",
        "",
        "![Screenshot.png](img.png)",
        "",
        "---",
        "",
        "[^1]: Footnote body that should not be narrated.",
      ].join("\n"),
    ),
  );
  expect(text).toContain("Opening");
  expect(text).toContain("Looking back on the first one. Watson said it felt strange.");
  expect(text).toContain("Tickets cost $5 to $10");
  expect(text).toContain("done!");
  expect(text).not.toContain("[C1]");
  expect(text).not.toContain("[C2]");
  expect(text).not.toContain("01:10");
  expect(text).not.toContain("[^1]");
  expect(text).not.toContain("Footnote body");
  expect(text).not.toContain("print(");
  expect(text).not.toContain("Margin");
  expect(text).not.toContain("\\frac");
  expect(text).not.toContain("\\sum");
  expect(text).not.toContain("https://");
  expect(text).not.toContain("Screenshot.png");
  expect(text).not.toContain("🔥");
  expect(text).not.toContain("→");
  expect(text).not.toContain("\u200B");
});

it("also sanitizes plain note nodes", () => {
  const note: SpaceCanvasNode = {
    id: "note",
    type: "note",
    initialPosition: { x: 0, y: 0 },
    data: {
      kind: "note",
      title: "Draft",
      plainText: "Key quote [C4] (02:15-02:30) ✅ https://knovum.dev\n---\nNext line.",
      tiptapContent: { type: "doc", content: [] },
    },
  };
  expect(narrationText(note)).toBe("Key quote\nNext line.");
});
