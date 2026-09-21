import { compact, minBy, take } from "es-toolkit";

import { sliceUtf16Safe, truncateUtf16 } from "../model/text";

export function searchTerms(value: string): string[] {
  return take(compact(value.trim().split(/\s+/u).map(cleanToken)), 8);
}

export function markSpaceSubstring(text: string, terms: readonly string[]): string {
  const lower = text.toLocaleLowerCase();
  const candidates = compact(
    terms.map((term) => {
      const index = lower.indexOf(term.toLocaleLowerCase());
      return index >= 0 ? { index, length: term.length } : undefined;
    }),
  );
  const match = minBy(candidates, (candidate) => candidate.index);
  if (!match) return truncateUtf16(text, 320);
  const start = Math.max(0, match.index - 120);
  const end = Math.min(text.length, match.index + match.length + 180);
  return `${start ? "… " : ""}${sliceUtf16Safe(text, start, match.index)}‹${sliceUtf16Safe(text, match.index, match.index + match.length)}›${sliceUtf16Safe(text, match.index + match.length, end)}${end < text.length ? " …" : ""}`;
}

function cleanToken(value: string) {
  return value.replace(/^["'“”‘’]+|["'“”‘’]+$/gu, "").trim();
}
