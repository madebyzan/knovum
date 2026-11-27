import { describe, expect, it } from "vitest";

import { extractMarkdownMermaidBlocks, extractTiptapMermaidBlocks } from "../export/diagrams";

describe("export mermaid diagram extraction", () => {
  it("extracts mermaid fenced blocks from markdown while ignoring other languages", () => {
    const md = [
      "# Research",
      "```ts",
      "const x = 1;",
      "```",
      "```mermaid",
      'flowchart TD\n  A["全球宏观"] --> B["中国宏观"]',
      "```",
      "~~~mermaid",
      "sequenceDiagram\n  A->>B: Hello",
      "~~~",
    ].join("\n");

    expect(extractMarkdownMermaidBlocks(md)).toEqual([
      'flowchart TD\n  A["全球宏观"] --> B["中国宏观"]',
      "sequenceDiagram\n  A->>B: Hello",
    ]);
  });

  it("extracts mermaid codeBlock nodes from tiptap note JSON", () => {
    const blocks = extractTiptapMermaidBlocks({
      type: "doc",
      content: [
        { type: "codeBlock", attrs: { language: "rust" }, content: [{ type: "text", text: "fn main() {}" }] },
        {
          type: "codeBlock",
          attrs: { language: "mermaid" },
          content: [{ type: "text", text: "graph LR\n  A --> B" }],
        },
      ],
    });
    expect(blocks).toEqual(["graph LR\n  A --> B"]);
  });
});
