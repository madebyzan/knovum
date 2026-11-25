import type { MarkdownIt as MarkdownItInstance, StateBlock, StateInline } from "markdown-it";

import { findInlineMathClose, isDigitChar, renderKatexHtml } from "./math-rendering";

function inlineMathRule(state: StateInline, silent: boolean): boolean {
  const { src, pos: start, posMax } = state;
  if (src.charCodeAt(start) !== 0x24 || src.charCodeAt(start + 1) === 0x24) return false;
  if (isDigitChar(src[start - 1])) return false;
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

function emitMultiLineBlockMath(
  state: StateBlock,
  firstLine: string,
  startLine: number,
  endLine: number,
): void {
  const close = findBlockMathClose(state, startLine, endLine);
  const stopLine = close ? close.line : endLine;
  const middle = state.getLines(startLine + 1, stopLine, state.tShift[startLine]!, true).trimEnd();
  const content = [firstLine, middle, close?.trailing ?? ""].filter(Boolean).join("\n").trim();
  pushBlockMath(state, content, startLine, close ? close.line + 1 : endLine);
}

function blockMathRule(state: StateBlock, startLine: number, endLine: number, silent: boolean): boolean {
  const startPos = state.bMarks[startLine]! + state.tShift[startLine]!;
  const maxPos = state.eMarks[startLine]!;
  if (startPos + 2 > maxPos || state.src.slice(startPos, startPos + 2) !== "$$") return false;
  if (state.src.charCodeAt(startPos + 2) === 0x24) return false;

  const firstLine = state.src.slice(startPos + 2, maxPos).trim();
  if (firstLine.endsWith("$$") && firstLine.length >= 2) {
    const content = firstLine.slice(0, -2).trim();
    if (!content) return false;
    if (!silent) pushBlockMath(state, content, startLine, startLine + 1);
    return true;
  }
  if (!silent) emitMultiLineBlockMath(state, firstLine, startLine, endLine);
  return true;
}

export function markdownMathPlugin(md: MarkdownItInstance): void {
  md.inline.ruler.after("escape", "math_inline", inlineMathRule);
  md.block.ruler.after("blockquote", "math_block", blockMathRule, {
    alt: ["paragraph", "reference", "blockquote", "list"],
  });
  md.renderer.rules.math_inline = (tokens, index) => renderKatexHtml(tokens[index]?.content ?? "", false);
  md.renderer.rules.math_block = (tokens, index) =>
    `<div class="ui-markdown-math-block">${renderKatexHtml(tokens[index]?.content ?? "", true)}</div>\n`;
}
