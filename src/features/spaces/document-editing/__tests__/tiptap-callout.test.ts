import { Markdown } from "@tiptap/markdown";
import { StarterKit } from "@tiptap/starter-kit";
import { Editor } from "@tiptap/vue-3";
import { describe, expect, it } from "vitest";

import { createCalloutExtension, normalizeCalloutKind } from "../tiptap-callout";

function createEditor(content = "") {
  return new Editor({
    extensions: [
      StarterKit,
      createCalloutExtension(false),
      Markdown.configure({ markedOptions: { gfm: true } }),
    ],
    content,
    contentType: "markdown",
  });
}

describe("GFM callout extension", () => {
  it("round-trips GFM alert blockquotes through Markdown", () => {
    const editor = createEditor("> [!WARNING]\n> Back up before migrating.\n");
    expect(editor.getJSON().content?.[0]).toMatchObject({
      type: "callout",
      attrs: { kind: "warning" },
    });
    const markdown = editor.getMarkdown().trim();
    expect(markdown).toBe("> [!WARNING]\n> Back up before migrating.");
    editor.destroy();
  });

  it("supports toggling callouts and normalizing unknown kinds to note", () => {
    expect(normalizeCalloutKind("IMPORTANT")).toBe("important");
    expect(normalizeCalloutKind("unknown")).toBe("note");

    const editor = createEditor("Keep this visible.");
    editor.commands.setTextSelection(1);
    expect(editor.commands.toggleCallout("tip")).toBe(true);
    expect(editor.isActive("callout", { kind: "tip" })).toBe(true);
    expect(editor.getMarkdown().trim()).toBe("> [!TIP]\n> Keep this visible.");

    expect(editor.commands.toggleCallout("warning")).toBe(true);
    expect(editor.isActive("callout", { kind: "warning" })).toBe(true);
    expect(editor.getMarkdown().trim()).toBe("> [!WARNING]\n> Keep this visible.");

    expect(editor.commands.toggleCallout("warning")).toBe(true);
    expect(editor.isActive("callout")).toBe(false);
    editor.destroy();
  });
});
