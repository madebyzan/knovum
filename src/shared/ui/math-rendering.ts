import { renderToString, type KatexOptions } from "katex";

const CURRENCY_ONLY = /^\d+(?:[.,]\d+)?$/u;
const ASCII_ALNUM = /[A-Za-z0-9]/;

export const KATEX_BASE_OPTIONS: KatexOptions = {
  throwOnError: false,
  strict: "ignore",
  output: "html",
  maxSize: 20,
};

export function renderKatexHtml(latex: string, displayMode: boolean): string {
  return renderToString(latex, {
    ...KATEX_BASE_OPTIONS,
    displayMode,
  });
}

export function isEscapedAt(src: string, pos: number): boolean {
  let backslashes = 0;
  for (let index = pos - 1; index >= 0 && src.charCodeAt(index) === 0x5c; index -= 1) {
    backslashes += 1;
  }
  return backslashes % 2 === 1;
}

export function isDigitChar(char: string | undefined): boolean {
  return Boolean(char && /\d/.test(char));
}

function isValidInlineMathClose(src: string, end: number): boolean {
  const prevChar = src[end - 1];
  const nextChar = src[end + 1];
  if (!prevChar || /\s/.test(prevChar)) return false;
  if (nextChar === "$" || (nextChar && ASCII_ALNUM.test(nextChar))) return false;
  return true;
}

/**
 * Finds the closing single `$` for inline math starting at `start`, staying on one line and
 * rejecting whitespace-padded delimiters, currency amounts, shell variables, and `$$`.
 */
export function findInlineMathClose(src: string, start: number, posMax: number): number {
  for (let end = start + 1; end < posMax; end += 1) {
    const code = src.charCodeAt(end);
    if (code === 0x0a || code === 0x0d) return -1;
    if (code !== 0x24 || isEscapedAt(src, end)) continue;
    // Unescaped `$` cannot occur inside inline math, so the first unescaped `$` must either
    // validly close this expression or reject `start` as an opening math delimiter.
    if (!isValidInlineMathClose(src, end)) return -1;
    const content = src.slice(start + 1, end).trim();
    return content && !CURRENCY_ONLY.test(content) ? end : -1;
  }
  return -1;
}

/** Matches a strict single-line `$latex$` expression starting at index 0 of `src`. */
export function matchInlineMathAtStart(src: string): { raw: string; latex: string } | undefined {
  if (src.charCodeAt(0) !== 0x24 || src.charCodeAt(1) === 0x24) return undefined;
  const nextChar = src[1];
  if (!nextChar || /\s/.test(nextChar)) return undefined;
  const end = findInlineMathClose(src, 0, src.length);
  if (end < 0) return undefined;
  return { raw: src.slice(0, end + 1), latex: src.slice(1, end).trim() };
}

/** Renders serialized Tiptap inline/block math placeholders into KaTeX HTML for read-only views. */
export function hydrateMathHtml(rawHtml: string): string {
  if (!rawHtml.includes('data-type="inline-math"') && !rawHtml.includes('data-type="block-math"')) {
    return rawHtml;
  }
  const doc = new DOMParser().parseFromString(rawHtml, "text/html");
  for (const element of doc.body.querySelectorAll('[data-type="inline-math"], [data-type="block-math"]')) {
    const latex = element.getAttribute("data-latex") ?? "";
    const isBlock = element.getAttribute("data-type") === "block-math";
    element.classList.add("tiptap-mathematics-render");
    element.innerHTML = renderKatexHtml(latex, isBlock);
  }
  return doc.body.innerHTML;
}
