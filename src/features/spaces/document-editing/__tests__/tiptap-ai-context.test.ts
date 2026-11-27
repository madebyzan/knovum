import { Markdown } from "@tiptap/markdown";
import { StarterKit } from "@tiptap/starter-kit";
import { Editor } from "@tiptap/vue-3";
import { describe, expect, it } from "vitest";

import { createSpaceNode } from "../../canvas/node";
import { extractEditorDocumentContext } from "../tiptap-ai-context";
import { findActiveMentionToken, stripMentionToken } from "../tiptap-ai-mention";
import { createSpaceNotePayload } from "../tiptap-content";
import { filterSpaceAiReferences, listSpaceAiReferenceCandidates } from "../tiptap-space-references";

describe("editor AI context and Space reference extraction", () => {
  it("extracts title, enclosing heading breadcrumb, and surrounding blocks", () => {
    const editor = new Editor({
      extensions: [StarterKit],
      content: "<h1>Architecture</h1><h2>Storage</h2><p>SQLite stores local rows.</p><p>Next paragraph.</p>",
    });
    const docText = editor.state.doc.textBetween(0, editor.state.doc.content.size, "\n", "\n");
    const targetStart = docText.indexOf("SQLite");
    expect(targetStart).toBeGreaterThan(0);

    const endOfStorage = editor.state.doc.content.size - 16;
    const context = extractEditorDocumentContext(
      editor,
      { from: endOfStorage - 10, to: endOfStorage },
      {
        documentTitle: "Design Doc",
        references: [{ nodeId: "ref-1", title: "Benchmark", excerpt: "Sub-millisecond reads." }],
      },
    );

    expect(context.documentTitle).toBe("Design Doc");
    expect(context.sectionHeading).toBe("Architecture > Storage");
    expect(context.precedingText).toContain("SQLite");
    expect(context.followingText).toContain("Next paragraph");
    expect(context.references).toEqual([
      { nodeId: "ref-1", title: "Benchmark", excerpt: "Sub-millisecond reads." },
    ]);
    editor.destroy();
  });

  it("preserves Markdown headings, lists, and fenced mermaid blocks in AI context", () => {
    const editor = new Editor({
      extensions: [StarterKit, Markdown],
      content:
        '<h1>11月份工作</h1><ol><li><p>中阿项目开发</p></li></ol><pre><code class="language-mermaid">flowchart LR\n  A[Start] --&gt; B[Result]</code></pre><p></p>',
    });
    const cursorPos = editor.state.doc.content.size - 1;
    const context = extractEditorDocumentContext(editor, { from: cursorPos, to: cursorPos });

    expect(context.precedingText).toContain("# 11月份工作");
    expect(context.precedingText).toContain("1. 中阿项目开发");
    expect(context.precedingText).toContain("```mermaid");
    expect(context.precedingText).toContain("flowchart LR");
    editor.destroy();
  });

  it("lists and filters non-current semantic Space nodes as reference candidates", () => {
    const current = createSpaceNode(
      createSpaceNotePayload("Current Doc", "Draft text"),
      { x: 0, y: 0 },
      "self",
    );
    const research = createSpaceNode(
      createSpaceNotePayload("Interview Transcript", "Key user insights on Markdown."),
      { x: 10, y: 10 },
      "ref-1",
    );
    const empty = createSpaceNode(createSpaceNotePayload("Blank", "   "), { x: 20, y: 20 }, "blank");

    const candidates = listSpaceAiReferenceCandidates([current, research, empty], "self");
    expect(candidates).toHaveLength(1);
    expect(candidates[0]?.nodeId).toBe("ref-1");
    expect(candidates[0]?.excerpt).toContain("Key user insights");

    expect(filterSpaceAiReferences(candidates, "Interview")).toHaveLength(1);
    expect(filterSpaceAiReferences(candidates, "Unrelated")).toHaveLength(0);
  });

  it("parses and strips active @ mention tokens inside the Knovum prompt input", () => {
    expect(findActiveMentionToken("@", 1)).toEqual({ start: 0, end: 1, query: "" });
    expect(findActiveMentionToken("Compare with @bench", 19)).toEqual({ start: 13, end: 19, query: "bench" });
    expect(findActiveMentionToken("user@example.com", 16)).toBeNull();

    const token = findActiveMentionToken("Summarize @spec briefly", 15)!;
    expect(stripMentionToken("Summarize @spec briefly", token)).toEqual({
      text: "Summarize briefly",
      caret: 10,
    });
  });
});
