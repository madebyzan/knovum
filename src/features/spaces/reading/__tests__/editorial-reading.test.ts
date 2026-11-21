// @vitest-environment jsdom
import { describe, expect, it } from "vitest";

import { highlightCodeLines, highlightContainerCodeBlocks, normalizeCodeLanguage } from "../code-highlighter";
import { resolveCodePreviewKind } from "../code-preview";
import { extractMarkdownHeadings, extractTiptapHeadings, noteReadingHtml } from "../note-reading-html";

describe("editorial reading helpers", () => {
  it("extracts ATX headings from Markdown while ignoring fenced code blocks", () => {
    const markdown = [
      "# Overview",
      "Some text.",
      "```bash",
      "# not a heading",
      "```",
      "## Findings",
      "### Details ###",
    ].join("\n");
    expect(extractMarkdownHeadings(markdown)).toEqual([
      { index: 0, level: 1, text: "Overview" },
      { index: 1, level: 2, text: "Findings" },
      { index: 2, level: 3, text: "Details" },
    ]);
  });

  it("renders sanitized Note HTML and extracts TipTap headings", () => {
    const content = {
      type: "doc",
      content: [
        { type: "heading", attrs: { level: 2 }, content: [{ type: "text", text: "Key Point" }] },
        {
          type: "paragraph",
          content: [{ type: "text", text: "Retrieval improves accuracy." }],
        },
      ],
    };
    expect(extractTiptapHeadings(content)).toEqual([{ index: 0, level: 2, text: "Key Point" }]);
    const html = noteReadingHtml(content);
    expect(html).toContain("<h2>Key Point</h2>");
    expect(html).toContain("<p>Retrieval improves accuracy.</p>");
  });

  it("normalizes code languages and highlights code lines and containers via Shiki", async () => {
    expect(normalizeCodeLanguage("language-ts")).toBe("typescript");
    expect(normalizeCodeLanguage(undefined, "main.rs")).toBe("rust");

    const lines = await highlightCodeLines("const x = 1;\nexport default x;", "ts", false);
    expect(lines).toHaveLength(2);
    expect(lines?.[0]).toContain("<span");
    expect(lines?.[0]).toContain("const");

    const host = document.createElement("div");
    host.innerHTML = '<pre><code class="language-ts">const y = 2;</code></pre>';
    document.body.append(host);
    try {
      await highlightContainerCodeBlocks(host, true);
      const code = host.querySelector("code")!;
      expect(code.dataset.shikiTheme).toBe("dark");
      expect(code.innerHTML).toContain("<span");
    } finally {
      host.remove();
    }
  });

  it("detects previewable HTML and SVG code nodes", () => {
    expect(resolveCodePreviewKind("html", "download.html")).toBe("html");
    expect(resolveCodePreviewKind(undefined, "diagram.svg")).toBe("svg");
    expect(resolveCodePreviewKind("typescript", "app.ts")).toBeUndefined();
  });
});
