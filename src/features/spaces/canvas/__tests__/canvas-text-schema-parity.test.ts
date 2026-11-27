// @vitest-environment jsdom
import { Editor, getSchema } from "@tiptap/core";
import { mount } from "@vue/test-utils";
import { describe, expect, it, vi } from "vitest";

import {
  createCanvasTextExtensions,
  createCanvasTextSchema,
  createSpaceTiptapExtensions,
} from "../../document-editing/tiptap-extensions";
import { tiptapContentSchema } from "../../model/nodes/schema/content";
import { canvasTextHtml } from "../presentation/nodes/canvas-text-html";
import SpaceCanvasRichText from "../presentation/nodes/SpaceCanvasRichText.vue";

vi.mock("@tauri-apps/api/core", () => ({
  convertFileSrc: (path: string) => `asset://localhost/${path}`,
}));

describe("canvas text and document editor schema parity", () => {
  it("renders and edits rich note content created in the document editor without throwing", () => {
    const richDocEditor = new Editor({
      extensions: createSpaceTiptapExtensions(
        "",
        () => undefined,
        () => undefined,
      ),
      content: {
        type: "doc",
        content: [
          {
            type: "heading",
            attrs: { level: 1, textAlign: "center" },
            content: [{ type: "text", text: "Section Title" }],
          },
          {
            type: "paragraph",
            content: [
              {
                type: "text",
                text: "Highlighted",
                marks: [{ type: "highlight", attrs: { color: "#fef08a" } }],
              },
              { type: "text", text: " H" },
              { type: "text", text: "2", marks: [{ type: "subscript" }] },
              { type: "text", text: "O" },
              { type: "text", text: "2", marks: [{ type: "superscript" }] },
            ],
          },
          { type: "horizontalRule" },
          {
            type: "codeBlock",
            attrs: { language: "ts" },
            content: [{ type: "text", text: "const x = 1;" }],
          },
          {
            type: "callout",
            attrs: { kind: "tip" },
            content: [{ type: "paragraph", content: [{ type: "text", text: "Callout note" }] }],
          },
          {
            type: "image",
            attrs: { assetPath: "assets/diagram.png", alt: "Diagram" },
          },
          {
            type: "table",
            content: [
              {
                type: "tableRow",
                content: [
                  {
                    type: "tableHeader",
                    content: [{ type: "paragraph", content: [{ type: "text", text: "Key" }] }],
                  },
                  {
                    type: "tableCell",
                    content: [{ type: "paragraph", content: [{ type: "text", text: "Val" }] }],
                  },
                ],
              },
            ],
          },
        ],
      },
    });
    const content = tiptapContentSchema.parse(richDocEditor.getJSON());
    richDocEditor.destroy();

    const html = canvasTextHtml(content, "note");
    expect(html).toContain("<h1");
    expect(html).toContain("Section Title");
    expect(html).toContain("<mark");
    expect(html).toContain("<sub>2</sub>");
    expect(html).toContain("<sup>2</sup>");
    expect(html).toContain("<hr>");
    expect(html).toContain("<pre>");
    expect(html).toContain('data-type="callout"');
    expect(html).toContain("Callout note");
    expect(html).toContain('src="asset://localhost/assets/diagram.png"');
    expect(html).toContain("<table");

    const wrapper = mount(SpaceCanvasRichText, {
      props: { content, empty: false, kind: "note", placeholder: "Write" },
    });
    expect(wrapper.find("h1").text()).toBe("Section Title");
    wrapper.unmount();

    const inlineEditor = new Editor({
      content,
      extensions: createCanvasTextExtensions("", "note"),
    });
    expect(inlineEditor.getJSON().content?.map((node) => node.type)).toEqual([
      "heading",
      "paragraph",
      "horizontalRule",
      "codeBlock",
      "callout",
      "image",
      "table",
    ]);
    inlineEditor.destroy();
  });

  it("preserves canvas fontSize marks when a note opens in the document editor", () => {
    const content = tiptapContentSchema.parse({
      type: "doc",
      content: [
        {
          type: "paragraph",
          content: [
            { type: "text", text: "Sized", marks: [{ type: "textStyle", attrs: { fontSize: "20pt" } }] },
          ],
        },
      ],
    });
    const docEditor = new Editor({
      content,
      extensions: createSpaceTiptapExtensions(
        "",
        () => undefined,
        () => undefined,
      ),
    });
    expect(docEditor.getHTML()).toContain("font-size: 20pt");
    docEditor.destroy();
  });

  it("keeps node and mark schemas structurally identical across document and canvas surfaces", () => {
    const docSchema = getSchema(
      createSpaceTiptapExtensions(
        "",
        () => undefined,
        () => undefined,
      ),
    );
    const canvasNoteSchema = createCanvasTextSchema("note");
    const canvasTextSchema = createCanvasTextSchema("text");
    const inlineNoteSchema = getSchema(createCanvasTextExtensions("", "note"));

    const docNodes = Object.keys(docSchema.nodes).toSorted();
    const docMarks = Object.keys(docSchema.marks).toSorted();

    expect(Object.keys(canvasNoteSchema.nodes).toSorted()).toEqual(docNodes);
    expect(Object.keys(canvasNoteSchema.marks).toSorted()).toEqual(docMarks);
    expect(Object.keys(canvasTextSchema.nodes).toSorted()).toEqual(docNodes);
    expect(Object.keys(canvasTextSchema.marks).toSorted()).toEqual(docMarks);
    expect(Object.keys(inlineNoteSchema.nodes).toSorted()).toEqual(docNodes);
    expect(Object.keys(inlineNoteSchema.marks).toSorted()).toEqual(docMarks);
  });
});
