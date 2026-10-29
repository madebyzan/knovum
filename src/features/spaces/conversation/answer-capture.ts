import type { SpaceMarkdownPayload } from "../model/nodes/node-payload";
import { truncateUtf16 } from "../model/text";

/** A passage selected in an answer: its words, and the citations of the blocks it touches. */
export interface AnswerPassage {
  text: string;
  citations: readonly number[];
}

const TITLE_UNITS = 80;
// `[C3]` resolves to a source only inside its answer; on the canvas it would point at nothing.
const CITATION_MARKERS = /[ \t]*\[C[1-9]\d*\]/gu;

export function withoutCitationMarkers(markdown: string) {
  return markdown.replace(CITATION_MARKERS, "");
}

/** The first line of an answer as a node title: its words, without Markdown syntax. */
export function answerTitle(markdown: string, fallback: string) {
  const line = withoutCitationMarkers(markdown)
    .split(/\r?\n/u)
    .map((candidate) => plainLine(candidate))
    .find(Boolean);
  return truncateUtf16(line ?? "", TITLE_UNITS) || fallback;
}

function plainLine(line: string) {
  return line
    .replace(/^\s*(?:#{1,6}\s+|>\s*|[-*+]\s+|\d+[.)]\s+)/u, "")
    .replace(/!?\[([^\]]*)\]\([^)]*\)/gu, "$1")
    .replace(/[*_`~]+/gu, "")
    .replace(/:$/u, "")
    .trim();
}

/** A whole answer as a Markdown node, keeping its headings, lists, tables and code. */
export function markdownFromAnswer(markdown: string, fallbackTitle: string): SpaceMarkdownPayload {
  return {
    kind: "markdown",
    title: answerTitle(markdown, fallbackTitle),
    text: withoutCitationMarkers(markdown).trim(),
  };
}

/** The draft with the selected passage quoted above it, so the next question says what it is about. */
export function quotedFollowUp(selection: string, draft: string) {
  const quote = selection
    .trim()
    .split(/\r?\n/u)
    .map((line) => (line.trim() ? `> ${line.trimEnd()}` : ">"))
    .join("\n");
  const rest = draft.trim();
  return rest ? `${quote}\n\n${rest}` : `${quote}\n\n`;
}
