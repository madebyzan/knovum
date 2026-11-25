// @vitest-environment jsdom
import { Editor } from "@tiptap/core";
import { describe, expect, it } from "vitest";

import { canvasTextHtml } from "../../canvas/presentation/nodes/canvas-text-html";
import { tiptapContentSchema } from "../../model/nodes/schema/content";
import { noteReadingHtml } from "../../reading/note-reading-html";
import { createCanvasTextExtensions } from "../tiptap-extensions";
import { tiptapPlainText } from "../tiptap-mathematics";

describe("tiptap math parsing, plain text, and read-only hydration", () => {
  it("parses strict LaTeX math while leaving currency and shell variables as plain text", () => {
    const editor = new Editor({
      extensions: createCanvasTextExtensions("", "note"),
      content: "Costs $5 and $10, env $HOME/$PATH, formula $E = mc^2$.\n\n$$\n\\sum_{i=1}^{n} x_i\n$$",
      contentType: "markdown",
    });
    const json = tiptapContentSchema.parse(editor.getJSON());
    const paragraph = json.content?.[0];
    const inlineMathNodes = paragraph?.content?.filter((node) => node.type === "inlineMath") ?? [];
    expect(inlineMathNodes).toEqual([{ type: "inlineMath", attrs: { latex: "E = mc^2" } }]);
    expect(json.content?.[1]).toEqual({
      type: "blockMath",
      attrs: { latex: "\\sum_{i=1}^{n} x_i" },
    });

    const plainText = tiptapPlainText(editor.state.doc);
    expect(plainText).toContain("Costs $5 and $10");
    expect(plainText).toContain("$HOME/$PATH");
    expect(plainText).toContain("E = mc^2");
    expect(plainText).toContain("\\sum_{i=1}^{n} x_i");

    const canvasHtml = canvasTextHtml(json, "note");
    const readerHtml = noteReadingHtml(json);
    expect(canvasHtml).toContain('class="katex"');
    expect(canvasHtml).toContain('class="katex-display"');
    expect(readerHtml).toContain('class="katex"');
    expect(readerHtml).toContain('class="katex-display"');

    editor.destroy();
  });
});
