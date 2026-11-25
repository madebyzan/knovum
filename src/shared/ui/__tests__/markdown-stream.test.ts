// @vitest-environment jsdom
// DOMPurify loses the first element of a fragment under happy-dom's parser; markdown output is
// asserted against a spec-conformant one.
import DOMPurify from "dompurify";
import { describe, expect, it, vi } from "vitest";

import { createMarkdownRenderer } from "../markdown-renderer";
import { createMarkdownStream } from "../markdown-stream";

// Every construct whose block structure depends on the lines after it: setext headings and
// tables reshape the paragraph above them, lists absorb indented and blank-separated items,
// fences swallow everything until they close, and reference links resolve against definitions.
const ANSWER = [
  "# 结论",
  "",
  "第一段，带 **加粗**、`代码` 和链接 https://example.com。",
  "第二行是同一段的延续。",
  "",
  "Setext 标题",
  "===",
  "",
  "| 方案 | 成本 |",
  "| --- | --- |",
  "| A | 低 |",
  "| B | 高 |",
  "",
  "- 第一项",
  "- 第二项",
  "",
  "  第二项的续段",
  "  - 子项",
  "",
  "1. 有序",
  "2. 列表",
  "",
  "> 引用第一行",
  "延续到引用里",
  "",
  "```ts",
  "const a = 1;",
  "",
  "const b = 2;",
  "```",
  "",
  "参考 [文档][docs]。",
  "",
  "[docs]: https://example.com/docs",
  "",
  "---",
  "",
  "    缩进代码",
  "",
  "结尾段落",
].join("\n");

function oneShot(source: string) {
  return DOMPurify.sanitize(createMarkdownRenderer().render(source));
}

function streamed(source: string, step: number) {
  const render = createMarkdownStream(createMarkdownRenderer());
  let frame = render("");
  for (let end = step; end < source.length + step; end += step) frame = render(source.slice(0, end));
  return frame;
}

describe("streamed markdown", () => {
  it.each([1, 3, 17])(
    "renders exactly what a whole-document render produces (%i chars per delta)",
    (step) => {
      const frame = streamed(ANSWER, step);

      expect(frame.settled.join("") + frame.open).toBe(oneShot(ANSWER));
    },
  );

  it("parses only the open block once earlier blocks are settled", () => {
    const md = createMarkdownRenderer();
    const parse = vi.spyOn(md, "parse");
    const render = createMarkdownStream(md);
    const settled = [...render(ANSWER).settled];

    const frame = render(`${ANSWER}，以及补充。`);

    expect(parse.mock.lastCall?.[0]).toBe("结尾段落，以及补充。");
    expect(frame.settled).toEqual(settled);
    expect(frame.open).toBe("<p>结尾段落，以及补充。</p>\n");
  });

  it("renders again when a reference definition gives an earlier link its target", () => {
    const render = createMarkdownStream(createMarkdownRenderer());
    render("See [docs][d].\n\nMore.\n\n");

    const frame = render("See [docs][d].\n\nMore.\n\n[d]: https://example.com\n\nEnd.");

    expect(frame.restarted).toBe(true);
    expect(frame.settled.join("")).toContain('<a href="https://example.com">docs</a>');
  });

  it("formats an unfinished block while it is still being written", () => {
    const render = createMarkdownStream(createMarkdownRenderer());

    expect(render("Intro.\n\n**bold** and `co").open).toBe("<p><strong>bold</strong> and `co</p>\n");
    expect(render("Intro.\n\n```ts\nconst a").open).toContain('<code class="language-ts">const a');
  });

  it("restarts when the source no longer extends what it settled", () => {
    const render = createMarkdownStream(createMarkdownRenderer());
    render("First answer.\n\nSecond block.");

    const frame = render("Another answer.");

    expect(frame.restarted).toBe(true);
    expect(frame.settled).toEqual([]);
    expect(frame.open).toBe("<p>Another answer.</p>\n");
  });

  it("streams math and footnotes identically to a one-shot render", () => {
    const source =
      "Formula $a^2 + b^2 = c^2$ with note[^1].\n\n$$\nx = \\frac{1}{2}\n$$\n\n[^1]: Footnote body.";
    const frame = streamed(source, 7);
    expect(frame.settled.join("") + frame.open).toBe(oneShot(source));
  });

  it("streams multi-line block math with blank lines and leading dashes without settling prematurely", () => {
    const source = "Intro.\n\n$$\na\n\n- b\n$$\n\nOutro.";
    for (const step of [1, 3, 5]) {
      const frame = streamed(source, step);
      expect(frame.settled.join("") + frame.open).toBe(oneShot(source));
    }
  });

  it("preserves currency amounts and shell variables as plain text", () => {
    const source = "Costs $5 and $10, range $5-$10, env $HOME/$PATH, and math $x + y$.";
    const rendered = oneShot(source);
    expect(rendered).toContain("$5 and $10");
    expect(rendered).toContain("$5-$10");
    expect(rendered).toContain("$HOME/$PATH");
    expect(rendered).toContain('class="katex"');
  });
});
