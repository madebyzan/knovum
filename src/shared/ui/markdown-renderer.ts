import DOMPurify from "dompurify";
import MarkdownIt, { type MarkdownIt as MarkdownItInstance } from "markdown-it";

export function createMarkdownRenderer(): MarkdownItInstance {
  return new MarkdownIt({
    html: false,
    linkify: true,
    breaks: false,
    typographer: true,
  });
}

export function sanitizeRenderedMarkdown(html: string): string {
  return DOMPurify.sanitize(html);
}
