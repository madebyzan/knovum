import { Editor } from "@tiptap/core";
import { StarterKit } from "@tiptap/starter-kit";
import { describe, expect, it } from "vitest";

import { createSpaceCodeBlockExtension } from "../tiptap-code-block";
import { handleCodeBlockIndent } from "../tiptap-code-block-indent";

function createCodeEditor(code: string) {
  return new Editor({
    extensions: [StarterKit.configure({ codeBlock: false }), createSpaceCodeBlockExtension(false)],
    content: `<pre><code>${code}</code></pre>`,
  });
}

describe("code block multi-line indent and outdent", () => {
  it("indents and outdents multiple selected lines while preserving selection", () => {
    const editor = createCodeEditor("const a = 1;\nconst b = 2;\nconst c = 3;");
    editor.commands.setTextSelection({ from: 1, to: 20 });

    expect(handleCodeBlockIndent(editor.view, false)).toBe(true);
    expect(editor.state.doc.firstChild?.textContent).toBe("  const a = 1;\n  const b = 2;\nconst c = 3;");

    expect(handleCodeBlockIndent(editor.view, true)).toBe(true);
    expect(editor.state.doc.firstChild?.textContent).toBe("const a = 1;\nconst b = 2;\nconst c = 3;");
    editor.destroy();
  });

  it("inserts two spaces at a collapsed cursor and outdents the current line", () => {
    const editor = createCodeEditor("let x = 1;");
    editor.commands.setTextSelection(1);

    expect(handleCodeBlockIndent(editor.view, false)).toBe(true);
    expect(editor.state.doc.firstChild?.textContent).toBe("  let x = 1;");

    expect(handleCodeBlockIndent(editor.view, true)).toBe(true);
    expect(editor.state.doc.firstChild?.textContent).toBe("let x = 1;");
    editor.destroy();
  });
});
