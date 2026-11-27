import { stripCitationMarkers } from "../knowledge/model/citation-markers";
import { truncateUtf16 } from "../model/text";

export interface FormatExportMarkdownOptions {
  /** Strips Knovum internal `[Cn]` citation markers so exported prose stays clean and standalone. */
  stripCitations?: boolean;
}

const LEADING_H1_PATTERN = /^\s*#[ \t]+(.+?)(?:\s+#+)?[ \t]*(?:\r?\n|$)/u;
const FENCE_LINE_PATTERN = /^[ \t]{0,3}(`{3,}|~{3,})/u;
const NBSP_ONLY_LINE_PATTERN = /^[ \t]*(?:&nbsp;|\u00a0)+[ \t]*$/u;

function nextFenceState(
  line: string,
  fence: { char: string; len: number } | null,
): { next: { char: string; len: number } | null; matched: boolean } {
  const match = FENCE_LINE_PATTERN.exec(line);
  if (!match) return { next: fence, matched: false };
  const marker = match[1] ?? "";
  if (!fence) return { next: { char: marker[0] ?? "", len: marker.length }, matched: true };
  const closes = marker[0] === fence.char && marker.length >= fence.len && line.trim() === marker;
  return { next: closes ? null : fence, matched: true };
}

export function cleanSerializedMarkdown(markdown: string): string {
  const cleaned: string[] = [];
  let fence: { char: string; len: number } | null = null;

  for (const line of markdown.split(/\r?\n/u)) {
    const state = nextFenceState(line, fence);
    fence = state.next;
    cleaned.push(!state.matched && !fence && NBSP_ONLY_LINE_PATTERN.test(line) ? "" : line);
  }

  return cleaned
    .join("\n")
    .replace(/\n{3,}/gu, "\n\n")
    .replace(/^\n+/u, "");
}

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
  const rawBody = options?.stripCitations ? stripCitationMarkers(markdown) : markdown;
  const body = cleanSerializedMarkdown(rawBody);
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
