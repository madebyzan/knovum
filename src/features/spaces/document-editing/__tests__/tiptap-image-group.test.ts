// @vitest-environment jsdom
import { Editor } from "@tiptap/core";
import { describe, expect, it } from "vitest";

import { createSpaceTiptapExtensions } from "../tiptap-extensions";
import {
  moveGroupImage,
  removeGroupImage,
  splitImageGroupIntoParagraphs,
} from "../tiptap-image-group-actions";

function createTestEditor(markdown?: string) {
  const editor = new Editor({
    extensions: createSpaceTiptapExtensions(
      "",
      () => undefined,
      () => undefined,
    ),
  });
  if (markdown !== undefined) editor.commands.setContent(markdown, { contentType: "markdown" });
  return editor;
}

describe("tiptap imageGroup auto-grouping and directive round-trip", () => {
  it("auto-groups consecutive standard markdown images and serializes back without ::: wrapper", () => {
    const editor = createTestEditor(
      "![Step 1](https://example.com/1.png)\n\n![Step 2](https://example.com/2.png)\n\n![Step 3](https://example.com/3.png)",
    );
    const json = editor.getJSON();
    expect(json.content?.[0]?.type).toBe("imageGroup");
    expect(json.content?.[0]?.attrs?.layout).toBe("auto");
    expect(json.content?.[0]?.content).toHaveLength(3);

    const out = editor.getMarkdown();
    expect(out).not.toContain(":::gallery");
    expect(out).toContain("![Step 1](https://example.com/1.png)");
    expect(out).toContain("![Step 3](https://example.com/3.png)");
    editor.destroy();
  });

  it("parses and round-trips explicit :::gallery{layout=...} directives", () => {
    const md =
      ':::gallery{layout="compare"}\n![Before](https://example.com/a.png)\n\n![After](https://example.com/b.png)\n:::';
    const editor = createTestEditor(md);
    const json = editor.getJSON();
    expect(json.content?.[0]?.type).toBe("imageGroup");
    expect(json.content?.[0]?.attrs?.layout).toBe("compare");
    expect(json.content?.[0]?.content).toHaveLength(2);

    const out = editor.getMarkdown();
    expect(out).toContain(':::gallery{layout="compare"}');
    expect(out).toContain("![Before](https://example.com/a.png)");
    expect(out).toContain("![After](https://example.com/b.png)");
    expect(out.trim().endsWith(":::")).toBe(true);
    editor.destroy();
  });

  it("supports reordering, unwrapping when reduced to one image, and splitting with paragraphs", () => {
    const editor = createTestEditor(
      "![First](https://example.com/1.png)\n\n![Second](https://example.com/2.png)",
    );
    expect(moveGroupImage(editor, () => 0, 0, 1)).toBe(true);
    expect(editor.state.doc.firstChild?.firstChild?.attrs.alt).toBe("Second");

    expect(splitImageGroupIntoParagraphs(editor, () => 0)).toBe(true);
    expect(
      editor
        .getJSON()
        .content?.slice(0, 3)
        .map((node) => node.type),
    ).toEqual(["image", "paragraph", "image"]);
    editor.destroy();

    const editor2 = createTestEditor(
      "![First](https://example.com/1.png)\n\n![Second](https://example.com/2.png)",
    );
    expect(removeGroupImage(editor2, () => 0, 0)).toBe(true);
    expect(editor2.getJSON().content?.[0]?.type).toBe("image");
    editor2.destroy();
  });
});
