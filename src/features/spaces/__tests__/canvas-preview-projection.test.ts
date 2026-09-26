import { describe, expect, it } from "vitest";

import type { CanvasNodePresentationContext } from "../canvas/contract/canvas-detail-profile";
import type { TiptapJsonNode } from "../editor/tiptap-document";
import { projectPreviewItems } from "../presentation/canvas/nodes/canvas-preview-budget";
import { projectMarkdownPreview } from "../presentation/canvas/nodes/markdown-preview-projection";
import { projectTiptapPreview } from "../presentation/canvas/nodes/tiptap-preview-projection";

const contexts = {
  compact: { viewport: { level: "compact", previewItemLimit: 2 }, mode: "preview" },
  summary: { viewport: { level: "summary", previewItemLimit: 4 }, mode: "preview" },
  full: { viewport: { level: "full", previewItemLimit: 10 }, mode: "preview" },
  interactive: { viewport: { level: "compact", previewItemLimit: 2 }, mode: "interactive" },
} as const satisfies Readonly<Record<string, CanvasNodePresentationContext>>;

describe("canvas preview projection", () => {
  it("uses the shared 2/4/10 density budget and lets interaction reveal all items", () => {
    const items = Array.from({ length: 12 }, (_, index) => index + 1);

    expect(projectPreviewItems(items, contexts.compact)).toEqual(items.slice(0, 2));
    expect(projectPreviewItems(items, contexts.summary)).toEqual(items.slice(0, 4));
    expect(projectPreviewItems(items, contexts.full)).toEqual(items.slice(0, 10));
    expect(projectPreviewItems(items, contexts.interactive)).toEqual(items);
  });

  it("projects Tiptap at root-block boundaries without changing the source document", () => {
    const document: TiptapJsonNode = {
      type: "doc",
      content: Array.from({ length: 12 }, (_, index) => ({
        type: "paragraph",
        content: [{ type: "text", text: `Block ${index + 1}` }],
      })),
    };

    expect(projectTiptapPreview(document, contexts.compact).content).toHaveLength(2);
    expect(projectTiptapPreview(document, contexts.summary).content).toHaveLength(4);
    expect(projectTiptapPreview(document, contexts.full).content).toHaveLength(10);
    expect(projectTiptapPreview(document, contexts.interactive)).toBe(document);
    expect(document.content).toHaveLength(12);
  });

  it("projects Markdown as complete root blocks rather than clipping rendered DOM lines", () => {
    const source = [
      "# Heading",
      "",
      "A paragraph with **formatting**.",
      "",
      "- first",
      "- second",
      "",
      "```ts",
      "const answer = 42;",
      "```",
      "",
      "> final block",
    ].join("\n");

    const compact = projectMarkdownPreview(source, 2);
    expect(compact.visibleBlockCount).toBe(2);
    expect(compact.remainingBlockCount).toBe(3);
    expect(compact.projectedSource).toContain("# Heading");
    expect(compact.projectedSource).toContain("A paragraph with **formatting**.");
    expect(compact.projectedSource).not.toContain("- first");

    const summary = projectMarkdownPreview(source, 4);
    expect(summary.visibleBlockCount).toBe(4);
    expect(summary.remainingBlockCount).toBe(1);
    expect(summary.projectedSource).toContain("const answer = 42;");
    expect(summary.projectedSource).not.toContain("> final block");
  });
});
