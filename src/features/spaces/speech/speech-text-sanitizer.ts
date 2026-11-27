import { stripCitationMarkers, unescapeCitationMarkers } from "../knowledge/model/citation-markers";

const FOOTNOTE_DEFINITION_PATTERN = /^\[\^[^\]]+\]:.*(?:\n(?:[ \t]{2,}|\t).*)*$/gmu;
const FOOTNOTE_REF_PATTERN = /[ \t]*\[\^[^\]]+\]/gu;
const DISPLAY_MATH_PATTERN = /\$\$[\s\S]*?\$\$/gu;
// Strip inline LaTeX only when it contains TeX commands or math syntax so currency like "$5 to $10" stays intact.
const INLINE_LATEX_PATTERN = /(?<!\\)\$(?!\s)(?=[^$\n]*(?:\\[a-zA-Z]+|[_^{}]))[^$\n]+?(?<!\s)\$/gu;
const BARE_URL_PATTERN = /\b(?:https?:\/\/|www\.|mailto:)[^\s<>()[\]{}"'，。！？；：、]+/giu;
const MEDIA_TIMESTAMP_RANGE_PATTERN =
  /[ \t]*[(（]\d{1,2}:\d{2}(?::\d{2})?\s*[–—-]\s*\d{1,2}:\d{2}(?::\d{2})?[)）]/gu;
const NUMERIC_CITATION_PATTERN = /[ \t]*\[\d+(?:\s*[,–—-]\s*\d+)*\]/gu;
const INVISIBLE_CHAR_PATTERN = /(?!\n|\t)\p{Cc}|\p{Cf}|\u{FE0E}|\u{FE0F}|\u{20E3}/gu;
const NON_SPEAKABLE_SYMBOL_PATTERN =
  /[\p{Extended_Pictographic}\u{1F1E6}-\u{1F1FF}\u2190-\u21FF\u2500-\u25FF\u2700-\u27BF]/gu;
const EMPTY_ENCLOSURE_PATTERN = /[ \t]*(?:\(\s*\)|（\s*）|\[\s*\]|【\s*】)/gu;
const SPACE_BEFORE_PUNCTUATION_PATTERN = /[ \t]+([,.;:!?，。；：！？、])/gu;
const SPEAKABLE_CHAR_PATTERN = /[\p{L}\p{N}]/u;

const NON_NARRATIVE_DOM_SELECTOR =
  "script,style,nav,header,footer,pre,figure,figcaption,img,svg,audio,video,iframe,table,hr,section.footnotes";

/** Cleans Markdown source before HTML conversion so citations, footnotes, and raw TeX never enter TTS. */
export function prepareSpeechMarkdown(markdown: string): string {
  const withoutCitations = stripCitationMarkers(unescapeCitationMarkers(markdown));
  return withoutCitations
    .replace(FOOTNOTE_DEFINITION_PATTERN, "")
    .replace(FOOTNOTE_REF_PATTERN, "")
    .replace(DISPLAY_MATH_PATTERN, "")
    .replace(INLINE_LATEX_PATTERN, "");
}

/** Removes non-narrative HTML nodes (code blocks, tables, figures, images, footnote superscripts). */
export function pruneNonNarrativeDom(document: Document): void {
  document.querySelectorAll(NON_NARRATIVE_DOM_SELECTOR).forEach((element) => element.remove());
  document.querySelectorAll("sup").forEach((element) => {
    const text = element.textContent?.trim() ?? "";
    if (!text || /^\[?\d+\]?$/u.test(text) || /^C[1-9]\d*$/u.test(text)) element.remove();
  });
}

/**
 * Normalizes extracted text into clean, speakable prose for TTS: strips residual citations,
 * bare URLs, parenthesized timestamp ranges, invisible/control characters, emojis, and
 * purely decorative lines while preserving natural punctuation and paragraph breaks.
 */
export function sanitizeSpokenText(raw: string): string {
  const cleaned = stripCitationMarkers(unescapeCitationMarkers(raw))
    .replace(/\r\n?/gu, "\n")
    .replace(FOOTNOTE_REF_PATTERN, "")
    .replace(NUMERIC_CITATION_PATTERN, "")
    .replace(MEDIA_TIMESTAMP_RANGE_PATTERN, "")
    .replace(BARE_URL_PATTERN, "")
    .replace(INVISIBLE_CHAR_PATTERN, "")
    .replace(NON_SPEAKABLE_SYMBOL_PATTERN, "")
    .replace(EMPTY_ENCLOSURE_PATTERN, "");

  const lines = cleaned.split("\n").map(cleanSpokenLine);
  const kept: string[] = [];
  for (const line of lines) {
    if (!line) {
      if (kept.length && kept[kept.length - 1] !== "") kept.push("");
      continue;
    }
    if (SPEAKABLE_CHAR_PATTERN.test(line)) kept.push(line);
  }
  while (kept.length && kept[kept.length - 1] === "") kept.pop();
  return kept.join("\n");
}

function cleanSpokenLine(line: string): string {
  return line
    .replace(/[ \t]{2,}/gu, " ")
    .replace(SPACE_BEFORE_PUNCTUATION_PATTERN, "$1")
    .trim();
}
