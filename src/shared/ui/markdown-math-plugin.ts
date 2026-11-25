import { renderToString } from "katex";
import type { MarkdownIt as MarkdownItInstance, StateBlock, StateInline } from "markdown-it";

const CURRENCY_ONLY = /^\d+(?:[.,]\d+)?$/;

function renderKatex(latex: string, displayMode: boolean): string {
  return renderToString(latex, {
    displayMode,
    throwOnError: false,
    strict: "ignore",
    output: "html",
  });
}

function isEscaped(src: string, pos: number): boolean {
  let backslashes = 0;
  for (let i = pos - 1; i >= 0 && src.charCodeAt(i) === 0x5c; i -= 1) backslashes += 1;
  return backslashes % 2 === 1;
}

function isDigit(char: string | undefined): boolean {
  return Boolean(char && /\d/.test(char));
}

function isValidInlineMathClose(src: string, end: number): boolean {
  const prevChar = src[end - 1];
  return Boolean(prevChar && !/\s/.test(prevChar) && !isDigit(src[end + 1]));
}

function findInlineMathClose(src: string, start: number, posMax: number): number {
  for (let end = start + 1; end < posMax; end += 1) {
    if (src.charCodeAt(end) !== 0x24 || isEscaped(src, end)) continue;
    if (!isValidInlineMathClose(src, end)) continue;
    const content = src.slice(start + 1, end).trim();
    if (content && !CURRENCY_ONLY.test(content)) return end;
  }
  return -1;
}

function inlineMathRule(state: StateInline, silent: boolean): boolean {
  const { src, pos: start, posMax } = state;
  if (src.charCodeAt(start) !== 0x24 || src.charCodeAt(start + 1) === 0x24) return false;
  if (isDigit(src[start - 1])) return false;
  const nextChar = src[start + 1];
  if (!nextChar || /\s/.test(nextChar)) return false;

  const end = findInlineMathClose(src, start, posMax);
  if (end < 0) return false;
  if (!silent) {
    const token = state.push("math_inline", "math", 0);
    token.markup = "$";
    token.content = src.slice(start + 1, end).trim();
  }
  state.pos = end + 1;
  return true;
}

function findBlockMathClose(
  state: StateBlock,
  startLine: number,
  endLine: number,
): { line: number; trailing: string } | undefined {
  for (let line = startLine + 1; line < endLine; line += 1) {
    const lineStart = state.bMarks[line]! + state.tShift[line]!;
    const trimmed = state.src.slice(lineStart, state.eMarks[line]!).trim();
    if (trimmed.endsWith("$$")) return { line, trailing: trimmed.slice(0, -2) };
  }
  return undefined;
}

function pushBlockMath(state: StateBlock, content: string, startLine: number, nextLine: number): void {
  const token = state.push("math_block", "math", 0);
  token.block = true;
  token.markup = "$$";
  token.content = content;
  token.map = [startLine, nextLine];
  state.line = nextLine;
}

function blockMathRule(state: StateBlock, startLine: number, endLine: number, silent: boolean): boolean {
  const startPos = state.bMarks[startLine]! + state.tShift[startLine]!;
  const maxPos = state.eMarks[startLine]!;
  if (startPos + 2 > maxPos || state.src.slice(startPos, startPos + 2) !== "$$") return false;

  const firstLine = state.src.slice(startPos + 2, maxPos).trim();
  if (firstLine.endsWith("$$") && firstLine.length >= 2) {
    const content = firstLine.slice(0, -2).trim();
    if (!content) return false;
    if (!silent) pushBlockMath(state, content, startLine, startLine + 1);
    return true;
  }

  const close = findBlockMathClose(state, startLine, endLine);
  if (!close) return false;
  if (!silent) {
    const middle = state.getLines(startLine + 1, close.line, state.tShift[startLine]!, true).trimEnd();
    const content = [firstLine, middle, close.trailing].filter(Boolean).join("\n").trim();
    pushBlockMath(state, content, startLine, close.line + 1);
  }
  return true;
}

export function markdownMathPlugin(md: MarkdownItInstance): void {
  md.inline.ruler.after("escape", "math_inline", inlineMathRule);
  md.block.ruler.after("blockquote", "math_block", blockMathRule, {
    alt: ["paragraph", "reference", "blockquote", "list"],
  });
  md.renderer.rules.math_inline = (tokens, index) => renderKatex(tokens[index]?.content ?? "", false);
  md.renderer.rules.math_block = (tokens, index) =>
    `<div class="ui-markdown-math-block">${renderKatex(tokens[index]?.content ?? "", true)}</div>\n`;
}
