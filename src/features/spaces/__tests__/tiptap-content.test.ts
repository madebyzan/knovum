import { Editor } from "@tiptap/core";
import { describe, expect, it } from "vitest";

import { canvasTextAppearanceStyle } from "../canvas/presentation/nodes/canvas-text-appearance";
import { canvasTextHtml } from "../canvas/presentation/nodes/canvas-text-html";
import { createSpaceNotePayload, createSpaceTextPayload } from "../document-editing/tiptap-content";
import type { TiptapJsonNode } from "../document-editing/tiptap-document";
import { createCanvasTextExtensions } from "../document-editing/tiptap-extensions";
import { validateSpaceNodePayload } from "../model/nodes/node-validation";
import { tiptapContentSchema } from "../model/nodes/schema/content";

describe("spaces Tiptap content", () => {
  it("uses black note text regardless of the requested text color", () => {
    expect(canvasTextAppearanceStyle("note", { textColor: "red", fill: "blue" }).color).toBe(
      "var(--color-black)",
    );
    expect(canvasTextAppearanceStyle("text", { textColor: "red" }).color).toBe("var(--color-content-red)");
  });

  it("keeps note text uncolored while preserving selection typography and pasted content", () => {
    const editor = new Editor({
      content: '<p><span style="color: red; font-size: 24pt"><strong>Note</strong></span></p>',
      extensions: createCanvasTextExtensions("", "note"),
    });
    expect(editor.extensionManager.extensions.some(({ name }) => name === "color")).toBe(false);
    expect(editor.getHTML()).not.toContain("color:");
    expect(editor.getHTML()).toContain("font-size: 24pt");
    expect(editor.getHTML()).toContain("<strong>");
    editor.commands.setTextSelection({ from: 1, to: 5 });
    editor.chain().setFontSize("18pt").run();
    expect(editor.getHTML()).toContain("font-size: 18pt");
    editor.destroy();
  });

  it("renders the same content with note or text color capabilities", () => {
    const content = tiptapContentSchema.parse({
      type: "doc",
      content: [
        {
          type: "paragraph",
          content: [
            {
              type: "text",
              text: "Note",
              marks: [{ type: "textStyle", attrs: { color: "red", fontSize: "24pt" } }],
            },
          ],
        },
      ],
    });
    expect(canvasTextHtml(content, "text")).toContain("color: red");
    expect(canvasTextHtml(content, "note")).not.toContain("color:");
    expect(canvasTextHtml(content, "note")).toContain("font-size: 24pt");
  });

  it("creates Notes as structured Tiptap content instead of Markdown strings", () => {
    const note = createSpaceNotePayload("Research", "First line\nSecond line");

    expect(note.kind).toBe("note");
    expect(note.plainText).toBe("First line\nSecond line");
    expect(note.tiptapContent).toEqual({
      type: "doc",
      content: [
        { type: "paragraph", content: [{ type: "text", text: "First line" }] },
        { type: "paragraph", content: [{ type: "text", text: "Second line" }] },
      ],
    });
  });

  it("deep-clones Tiptap content and JSON attributes", () => {
    const tiptapContent: TiptapJsonNode = {
      type: "doc",
      content: [
        { type: "paragraph", attrs: { meta: { priority: 2 } }, content: [{ type: "text", text: "Idea" }] },
      ],
    };
    const cloned = tiptapContentSchema.parse(tiptapContent);
    const paragraph = cloned.content?.[0];

    expect(cloned).toEqual(tiptapContent);
    expect(cloned).not.toBe(tiptapContent);
    expect(paragraph).not.toBe(tiptapContent.content?.[0]);
    expect(paragraph?.attrs).not.toBe(tiptapContent.content?.[0]?.attrs);
  });

  it("preserves selection-level typography marks in canonical content", () => {
    const tiptapContent: TiptapJsonNode = {
      type: "doc",
      content: [
        {
          type: "paragraph",
          content: [
            {
              type: "text",
              text: "Selected",
              marks: [{ type: "bold" }, { type: "textStyle", attrs: { color: "#2563eb", fontSize: "24pt" } }],
            },
            { type: "text", text: " default" },
          ],
        },
      ],
    };

    expect(tiptapContentSchema.parse(tiptapContent)).toEqual(tiptapContent);
  });

  it("formats only the active selection and renders the same canonical tree", () => {
    const editor = new Editor({
      content: createSpaceTextPayload("Selected default", "Selected default").tiptapContent,
      extensions: createCanvasTextExtensions(""),
    });

    editor.commands.setTextSelection({ from: 1, to: 9 });
    editor.chain().setColor("#2563eb").setFontSize("24pt").toggleBold().run();
    const content = tiptapContentSchema.parse(editor.getJSON());
    editor.destroy();

    const html = canvasTextHtml(content);
    expect(html).toContain("color: #2563eb");
    expect(html).toContain("font-size: 24pt");
    expect(html).toContain("<strong>");
    expect(html).toContain("Selected");
    expect(html).toContain(" default");
  });

  it("enforces separate canonical contracts for Note and Markdown", () => {
    expect(validateSpaceNodePayload(createSpaceNotePayload("Note"))).toMatchObject({ kind: "note" });
    expect(validateSpaceNodePayload({ kind: "markdown", title: "Readme", text: "# Hello" })).toMatchObject({
      kind: "markdown",
    });
    expect(() => validateSpaceNodePayload({ kind: "note", title: "Legacy", text: "# old" })).toThrow(
      "Invalid input",
    );
    expect(() =>
      validateSpaceNodePayload({ kind: "markdown", title: "Wrong", tiptapContent: { type: "doc" } }),
    ).toThrow("Invalid input");
  });

  it("models free canvas text independently from sticky notes", () => {
    expect(validateSpaceNodePayload(createSpaceTextPayload("Heading", "Heading"))).toMatchObject({
      kind: "text",
      plainText: "Heading",
    });
    expect(createSpaceTextPayload("Text")).toMatchObject({
      plainText: "",
      tiptapContent: { type: "doc", content: [{ type: "paragraph" }] },
    });
  });

  it("keeps sensitive node values out of the SQLite item payload", () => {
    expect(
      validateSpaceNodePayload({
        kind: "credential",
        title: "Production API",
        credentialKind: "api-key",
        credentialConfigured: true,
      }),
    ).toMatchObject({ kind: "credential", credentialKind: "api-key" });
    expect(() =>
      validateSpaceNodePayload({
        kind: "credential",
        title: "Unsafe",
        credentialKind: "api-key",
        text: "sk-should-never-be-here",
      }),
    ).toThrow("Unrecognized key");
  });
});
