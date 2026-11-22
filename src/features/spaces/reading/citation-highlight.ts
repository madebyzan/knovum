import { findNormalizedTextRange } from "../knowledge/model/quote-match";

const BLOCK_SELECTOR = "h1, h2, h3, h4, p, li, blockquote, pre, td, th";
const MARK_ATTR = "data-citation-mark";
const HIT_ATTR = "data-reader-citation-hit";
const PULSE_ATTR = "data-reader-citation-pulse";
const MAX_HIGHLIGHTED_BLOCKS = 6;

function stripMarkdownLine(line: string): string {
  return line
    .trim()
    .replace(/^#{1,6}\s+/u, "")
    .replace(/^>\s?/u, "")
    .replace(/^(?:[-*+]|\d+\.)\s+/u, "")
    .replace(/[*_`~]/gu, "")
    .replace(/\s+/gu, " ")
    .trim();
}

function normalizeCitationText(value: string): string {
  return value
    .split(/\r?\n/u)
    .map(stripMarkdownLine)
    .filter(Boolean)
    .join(" ")
    .replace(/\s+/gu, " ")
    .trim()
    .toLowerCase();
}

function quoteClauses(quote: string): string[] {
  const cleaned = quote
    .split(/\r?\n/u)
    .map(stripMarkdownLine)
    .filter((line) => line.length >= 2);
  if (cleaned.length) return cleaned;
  const single = normalizeCitationText(quote);
  return single ? [single] : [];
}

/** Removes any previous citation highlights and unwraps inline `<mark>` tags cleanly. */
export function clearCitationHighlights(container: HTMLElement | null): void {
  if (!container) return;
  for (const mark of container.querySelectorAll<HTMLElement>(`mark[${MARK_ATTR}]`)) {
    const parent = mark.parentNode;
    if (!parent) continue;
    while (mark.firstChild) parent.insertBefore(mark.firstChild, mark);
    parent.removeChild(mark);
    parent.normalize();
  }
  for (const el of container.querySelectorAll<HTMLElement>(`[${HIT_ATTR}], [${PULSE_ATTR}]`)) {
    el.removeAttribute(HIT_ATTR);
    el.removeAttribute(PULSE_ATTR);
  }
}

function wrapFirstTextMatch(block: HTMLElement, clauses: readonly string[]): HTMLElement | null {
  const walker = document.createTreeWalker(block, NodeFilter.SHOW_TEXT);
  const textNodes: Text[] = [];
  for (let current = walker.nextNode(); current; current = walker.nextNode()) {
    if (current.textContent?.trim()) textNodes.push(current as Text);
  }
  for (const clause of clauses) {
    if (clause.length < 3) continue;
    for (const node of textNodes) {
      const value = node.textContent ?? "";
      const match = findNormalizedTextRange(value, clause);
      if (!match) continue;
      const range = document.createRange();
      range.setStart(node, match.start);
      range.setEnd(node, match.end);
      const mark = document.createElement("mark");
      mark.setAttribute(MARK_ATTR, "");
      range.surroundContents(mark);
      return mark;
    }
  }
  return null;
}

function scopeByHeading(
  blocks: HTMLElement[],
  heading?: string,
): { pool: HTMLElement[]; headingEl?: HTMLElement | undefined } {
  const targetHeading = heading?.trim().toLowerCase();
  if (!targetHeading) return { pool: blocks };
  const idx = blocks.findIndex(
    (el) => /^H[1-4]$/u.test(el.tagName) && normalizeCitationText(el.textContent ?? "") === targetHeading,
  );
  if (idx < 0) return { pool: blocks };
  return { pool: blocks.slice(idx), headingEl: blocks[idx] };
}

function blockMatchesClauses(text: string, normalizedClauses: readonly string[]): boolean {
  return normalizedClauses.some(
    (clause) =>
      text === clause ||
      (clause.length >= 4 && text.includes(clause)) ||
      (text.length >= 4 && clause.includes(text)),
  );
}

function matchContiguousBlocks(pool: HTMLElement[], clauses: readonly string[]): HTMLElement[] {
  const normalizedClauses = clauses.map((c) => c.toLowerCase());
  const fullNormalized = normalizedClauses.join(" ");
  const exactBlock = pool.find((el) => normalizeCitationText(el.textContent ?? "").includes(fullNormalized));
  if (exactBlock) return [exactBlock];

  const hits: HTMLElement[] = [];
  for (const el of pool) {
    const text = normalizeCitationText(el.textContent ?? "");
    if (!text) continue;
    if (blockMatchesClauses(text, normalizedClauses)) {
      hits.push(el);
      if (hits.length >= MAX_HIGHLIGHTED_BLOCKS) break;
    } else if (hits.length > 0) {
      break;
    }
  }
  return hits;
}

function matchBlocks(blocks: HTMLElement[], heading?: string, quote?: string): HTMLElement[] {
  const { pool, headingEl } = scopeByHeading(blocks, heading);
  const clauses = quote?.trim() ? quoteClauses(quote) : [];
  if (clauses.length) {
    const hits = matchContiguousBlocks(pool, clauses);
    if (hits.length) return hits;
  }
  return headingEl ? [headingEl] : [];
}

function markHighlightedBlocks(hits: readonly HTMLElement[], clauses: readonly string[]): HTMLElement {
  let primaryMark: HTMLElement | null = null;
  for (const hit of hits) {
    hit.setAttribute(HIT_ATTR, "");
    hit.setAttribute(PULSE_ATTR, "");
    if (!primaryMark && clauses.length) primaryMark = wrapFirstTextMatch(hit, clauses);
  }
  return primaryMark ?? hits[0]!;
}

/**
 * Highlights and smoothly scrolls to the cited passage inside a rendered prose container
 * (Markdown or Note HTML), handling multi-line Markdown chunk quotes and inline `<mark>` cues.
 */
export function applyProseCitationHighlight(
  container: HTMLElement | null,
  options: { heading?: string | undefined; quote?: string | undefined },
): HTMLElement | null {
  clearCitationHighlights(container);
  const heading = options.heading?.trim();
  const quote = options.quote?.trim();
  if (!container || (!heading && !quote)) return null;
  const blocks = [...container.querySelectorAll<HTMLElement>(BLOCK_SELECTOR)];
  const hits = matchBlocks(blocks, heading, quote);
  if (!hits.length) return null;
  const scrollTarget = markHighlightedBlocks(hits, quote ? quoteClauses(quote) : []);
  scrollTarget.scrollIntoView({ behavior: "smooth", block: "center" });
  return scrollTarget;
}
