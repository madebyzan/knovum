import { Editor } from "@tiptap/core";
import { afterEach, describe, expect, it } from "vitest";

import { createCanvasTextExtensions, createSpaceTiptapExtensions } from "../tiptap-extensions";

interface PastePayload {
  plain?: string;
  html?: string;
  vscodeMode?: string;
  files?: File[];
}

const editors: Editor[] = [];

function createEditor(content = "", surface: "document" | "canvas" = "document"): Editor {
  const extensions =
    surface === "document"
      ? createSpaceTiptapExtensions(
          "",
          () => undefined,
          () => undefined,
        )
      : createCanvasTextExtensions("", "note");
  const editor = new Editor({ content, extensions });
  editors.push(editor);
  return editor;
}

function dispatchPaste(editor: Editor, payload: PastePayload): ClipboardEvent {
  const event = new Event("paste", { bubbles: true, cancelable: true }) as ClipboardEvent;
  const files = payload.files ?? [];
  Object.defineProperty(event, "clipboardData", {
    value: {
      files,
      getData: (type: string) => {
        if (type === "text/plain") return payload.plain ?? "";
        if (type === "text/html") return payload.html ?? "";
        if (type === "vscode-editor-data" && payload.vscodeMode)
          return JSON.stringify({ mode: payload.vscodeMode });
        return "";
      },
    },
  });
  editor.view.dom.dispatchEvent(event);
  return event;
}

function pasteInto(payload: PastePayload, surface: "document" | "canvas" = "document") {
  const editor = createEditor("", surface);
  const event = dispatchPaste(editor, payload);
  return { editor, event };
}

function nonEmptyBlockTypes(editor: Editor): (string | undefined)[] {
  return (editor.getJSON().content ?? [])
    .filter((node) => node.type !== "paragraph" || Boolean(node.content?.length))
    .map((node) => node.type);
}

afterEach(() => {
  while (editors.length) editors.pop()?.destroy();
});

describe("MarkdownPaste", () => {
  it("renders pasted multi-section Markdown into structured Tiptap nodes with atomic undo", () => {
    const { editor, event } = pasteInto({
      plain: [
        "## 研究目的",
        "",
        "我希望利用这段英文访谈，提取**有价值**的信息。",
        "",
        "### 1. 访谈内容",
        "",
        "- Emma Watson 对童年出演哈利·波特的回忆。",
        "- 她对自己小时候的外貌和声音的感受。",
        "",
        "1. 一份完整的视频内容分析。",
        "2. 一份英语口语表达学习笔记。",
      ].join("\n"),
    });

    expect(event.defaultPrevented).toBe(true);
    expect(nonEmptyBlockTypes(editor)).toEqual([
      "heading",
      "paragraph",
      "heading",
      "bulletList",
      "orderedList",
    ]);
    expect(editor.getHTML()).not.toContain("## ");
    expect(editor.getHTML()).toContain("<strong>有价值</strong>");
    expect(editor.commands.undo()).toBe(true);
    expect(editor.isEmpty).toBe(true);
  });

  it("parses Markdown copied from code editors and defers non-Markdown snippets to codeBlock", () => {
    const markdown = pasteInto(
      {
        plain: "## Title\n\n- First item\n- Second item",
        html: '<div style="font-family:Menlo"><b>## Title</b><br><span>- First item</span></div>',
        vscodeMode: "markdown",
      },
      "canvas",
    );
    expect(markdown.event.defaultPrevented).toBe(true);
    expect(nonEmptyBlockTypes(markdown.editor)).toEqual(["heading", "bulletList"]);

    const python = pasteInto({ plain: "# Python comment\nitems = [1, 2]", vscodeMode: "python" });
    expect(nonEmptyBlockTypes(python.editor)).toEqual(["codeBlock"]);
  });

  it("inserts single-line inline Markdown without splitting the current paragraph", () => {
    const editor = createEditor("<p>这是结论</p>");
    editor.commands.setTextSelection(3);
    const event = dispatchPaste(editor, { plain: "**核心**" });

    expect(event.defaultPrevented).toBe(true);
    expect(editor.getJSON().content).toHaveLength(1);
    expect(editor.getHTML()).toContain("这是<strong>核心</strong>结论");
  });

  it("normalizes non-breaking spaces and BOM copied from macOS apps before parsing Markdown", () => {
    const { editor, event } = pasteInto({
      plain: "\uFEFF##\u00A0研究目的\u2028\u2028-\u00A0第一条\n-\u00A0第二条",
    });

    expect(event.defaultPrevented).toBe(true);
    expect(nonEmptyBlockTypes(editor)).toEqual(["heading", "bulletList"]);
    expect(editor.getHTML()).not.toContain("##");
  });

  it("parses raw Markdown inside plain ProseMirror slices or code-viewer tables while preserving rich HTML", () => {
    const plainSlice = pasteInto({
      plain: "## 研究目的\n\n- 第一条\n- 第二条",
      html: '<p data-pm-slice="1 1 []">## 研究目的</p><p>- 第一条</p><p>- 第二条</p>',
    });
    expect(nonEmptyBlockTypes(plainSlice.editor)).toEqual(["heading", "bulletList"]);

    const codeTable = pasteInto({
      plain: "## 研究目的\n\n- 第一条",
      html: '<table><tr><td data-line-number="1"></td><td>## 研究目的</td></tr><tr><td data-line-number="2"></td><td>- 第一条</td></tr></table>',
    });
    expect(nonEmptyBlockTypes(codeTable.editor)).toEqual(["heading", "bulletList"]);

    const richTable = pasteInto({
      plain: "## Heading\tValue",
      html: "<table><thead><tr><th>## Heading</th><th>Value</th></tr></thead></table>",
    });
    expect(nonEmptyBlockTypes(richTable.editor)).toEqual(["table"]);
  });

  it("defers to ProseMirror for formatted slices, rich HTML, plain prose, bare URLs, and code blocks", () => {
    const richSlice = pasteInto({ plain: "## Heading", html: '<h2 data-pm-slice="1 1 []">Heading</h2>' });
    expect(richSlice.editor.getHTML()).toContain("<h2>Heading</h2>");

    const richHtml = pasteInto({ plain: "## Heading", html: "<h2>Heading</h2><p><strong>Bold</strong></p>" });
    expect(richHtml.editor.getHTML()).toContain("<h2>Heading</h2>");
    expect(richHtml.editor.getHTML()).toContain("<strong>Bold</strong>");

    expect(nonEmptyBlockTypes(pasteInto({ plain: "普通纯文本段落，没有 Markdown 语法。" }).editor)).toEqual([
      "paragraph",
    ]);
    expect(pasteInto({ plain: "https://knovum.app/docs" }).editor.getText()).toContain(
      "https://knovum.app/docs",
    );

    const codeEditor = createEditor("<pre><code></code></pre>");
    codeEditor.commands.setTextSelection(1);
    dispatchPaste(codeEditor, { plain: "## Not a heading" });
    expect(nonEmptyBlockTypes(codeEditor)).toEqual(["codeBlock"]);
    expect(codeEditor.getText()).toContain("## Not a heading");
  });

  it("strips unsafe link protocols and renders GFM tables, blockquotes, task lists, and fenced code", () => {
    const unsafe = pasteInto({ plain: "[click](javascript:alert(1))" });
    expect(unsafe.editor.getHTML()).not.toContain("javascript:");
    expect(unsafe.editor.getText()).toContain("click");

    const gfm = pasteInto({
      plain:
        "> 一段引用说明\n\n| 栏目 | 状态 |\n| --- | --- |\n| 视频分析 | 完成 |\n\n- [x] 已验证\n- [ ] 待补充\n\n```ts\nconst count = 42;\n```",
    });
    expect(gfm.event.defaultPrevented).toBe(true);
    expect(nonEmptyBlockTypes(gfm.editor)).toEqual(["blockquote", "table", "taskList", "codeBlock"]);
    expect(gfm.editor.getHTML()).toContain("const count = 42;");
  });
});
