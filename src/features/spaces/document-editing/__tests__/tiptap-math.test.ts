import { Editor } from "@tiptap/core";
import { describe, expect, it, vi } from "vitest";

import { createSpaceTiptapExtensions } from "../tiptap-extensions";
import { insertAndEditMath, validateKatexSyntax, type MathEditTarget } from "../tiptap-math";
import { convertSelectionToInlineMath } from "../tiptap-math-rules";

function createEditor(content: string, onEditMath?: (target: MathEditTarget) => void) {
  return new Editor({
    extensions: createSpaceTiptapExtensions(
      "",
      () => undefined,
      () => undefined,
      onEditMath ? { onEditMath } : {},
    ),
    content,
    contentType: "markdown",
  });
}

function readLatex(node: unknown): unknown {
  if (!node || typeof node !== "object" || !("attrs" in node)) return undefined;
  const attrs = (node as { attrs?: Record<string, unknown> }).attrs;
  return attrs?.latex;
}

describe("tiptap math (KaTeX live in-place editing)", () => {
  it("parses and serializes inline and block LaTeX math through Markdown", () => {
    const editor = createEditor(
      "Energy is $E = mc^2$ in relativity.\n\n$$\n\\int_0^1 x^2 \\, dx = \\frac{1}{3}\n$$",
    );
    const json = editor.getJSON();
    const inlineMath = json.content?.[0]?.content?.find((node) => node.type === "inlineMath");
    const blockMath = json.content?.find((node) => node.type === "blockMath");

    expect(readLatex(inlineMath)).toBe("E = mc^2");
    expect(readLatex(blockMath)).toContain("\\int_0^1");

    const markdown = editor.getMarkdown();
    expect(markdown).toContain("$E = mc^2$");
    expect(markdown).toContain("$$");
    editor.destroy();
  });

  it("inserts an empty block equation placeholder and triggers live editing callback", () => {
    const onEdit = vi.fn<(target: MathEditTarget) => void>();
    const editor = createEditor("");
    expect(insertAndEditMath(editor, "block", onEdit)).toBe(true);
    expect(editor.getJSON().content?.[0]?.type).toBe("blockMath");
    expect(onEdit).toHaveBeenCalledWith(
      expect.objectContaining({ kind: "block", latex: "", createdEmpty: true }),
    );
    editor.destroy();
  });

  it("converts selected prose text into inline math and validates LaTeX syntax", () => {
    const onEdit = vi.fn<(target: MathEditTarget) => void>();
    const editor = createEditor("Formula E = mc^2 end");
    editor.commands.setTextSelection({ from: 9, to: 17 });
    expect(convertSelectionToInlineMath(editor, onEdit)).toBe(true);
    const inlineMath = editor.getJSON().content?.[0]?.content?.find((node) => node.type === "inlineMath");
    expect(readLatex(inlineMath)).toBe("E = mc^2");
    expect(onEdit).toHaveBeenCalledWith(expect.objectContaining({ kind: "inline", latex: "E = mc^2" }));

    expect(validateKatexSyntax("E = mc^2", false)).toBeNull();
    expect(validateKatexSyntax("\\frac{1}{", true)).not.toBeNull();
    editor.destroy();
  });
});
