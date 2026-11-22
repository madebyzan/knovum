import { stripCitationMarkers } from "../knowledge/model/citation-markers";
import { truncateUtf16 } from "../model/text";

export interface FormatExportMarkdownOptions {
  /** Strips Knovum internal `[Cn]` citation markers so exported prose stays clean and standalone. */
  stripCitations?: boolean;
}

const LEADING_H1_PATTERN = /^\s*#[ \t]+(.+?)(?:\s+#+)?[ \t]*(?:\r?\n|$)/u;

export function extractImportedMarkdownDocument(name: string, rawText: string) {
  const fallbackTitle = markdownTitle(name);
  const match = LEADING_H1_PATTERN.exec(rawText);
  if (!match) return { title: fallbackTitle, text: rawText };

  const headingText = match[1]?.trim();
  if (!headingText) return { title: fallbackTitle, text: rawText };

  const remainingText = rawText.slice(match[0].length).replace(/^(?:\r?\n)+/u, "");
  return {
    title: truncateUtf16(headingText, 512),
    text: remainingText,
  };
}

export function formatExportMarkdown(
  title: string,
  markdown: string,
  options?: FormatExportMarkdownOptions,
): string {
  const body = options?.stripCitations ? stripCitationMarkers(markdown) : markdown;
  const trimmedTitle = title.trim();
  const trimmedMarkdown = body.trim();
  if (!trimmedTitle) return body;
  if (/^#\s+/u.test(trimmedMarkdown)) return body;
  return trimmedMarkdown ? `# ${trimmedTitle}\n\n${body.trimStart()}` : `# ${trimmedTitle}\n`;
}

function markdownTitle(name: string) {
  const baseName = name.replace(/\.(?:md|markdown)$/iu, "") || name;
  return truncateUtf16(baseName, 512);
}
