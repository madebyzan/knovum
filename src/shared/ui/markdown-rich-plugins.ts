import type { MarkdownIt, StateBlock, StateCore, StateInline } from "markdown-it";

import { registerImageGroupMarkdownPlugin } from "./markdown-image-groups";
import { renderMarkdownMediaHtml } from "./markdown-media";

const CALLOUT_PREFIX = /^\[!(NOTE|TIP|IMPORTANT|WARNING|CAUTION)\](?:\s*\n|\s+|$)/iu;
const TASK_PREFIX = /^\[([ xX])\]\s+/u;

function escapeAttr(value: string): string {
  return value.replaceAll("&", "&amp;").replaceAll('"', "&quot;").replaceAll("<", "&lt;");
}

function isLikelyInlineLatex(candidate: string): boolean {
  if (!candidate || /^\s|\s$/u.test(candidate)) return false;
  return !/^\d+(?:[.,]\d+)?$/u.test(candidate);
}

function parseBlockMath(state: StateBlock, startLine: number, endLine: number, silent: boolean): boolean {
  const startPos = state.bMarks[startLine]! + state.tShift[startLine]!;
  const maxPos = state.eMarks[startLine]!;
  if (startPos + 2 > maxPos || state.src.slice(startPos, startPos + 2) !== "$$") return false;

  const restOfFirstLine = state.src.slice(startPos + 2, maxPos).trim();
  if (restOfFirstLine.endsWith("$$") && restOfFirstLine.length > 2) {
    if (silent) return true;
    const token = state.push("math_block", "div", 0);
    token.content = restOfFirstLine.slice(0, -2).trim();
    token.map = [startLine, startLine + 1];
    state.line = startLine + 1;
    return true;
  }

  let nextLine = startLine + 1;
  let found = false;
  while (nextLine < endLine) {
    const lineStart = state.bMarks[nextLine]! + state.tShift[nextLine]!;
    const lineEnd = state.eMarks[nextLine]!;
    if (state.src.slice(lineStart, lineEnd).trim() === "$$") {
      found = true;
      break;
    }
    nextLine += 1;
  }
  if (!found) return false;
  if (silent) return true;

  const content = state.getLines(startLine + 1, nextLine, state.tShift[startLine]!, true).trim();
  const token = state.push("math_block", "div", 0);
  token.content = content;
  token.map = [startLine, nextLine + 1];
  state.line = nextLine + 1;
  return true;
}

function findClosingInlineDollar(src: string, start: number, posMax: number): number {
  for (let end = start; end < posMax; end += 1) {
    const ch = src.charCodeAt(end);
    if (ch === 0x0a) return -1;
    if (ch === 0x24 && src.charCodeAt(end - 1) !== 0x5c) return end;
  }
  return -1;
}

function parseInlineMath(state: StateInline, silent: boolean): boolean {
  const { src, pos, posMax } = state;
  if (src.charCodeAt(pos) !== 0x24 || src.charCodeAt(pos + 1) === 0x24) return false;
  if (pos > 0 && src.charCodeAt(pos - 1) === 0x5c) return false;
  const end = findClosingInlineDollar(src, pos + 1, posMax);
  if (end === -1 || src.charCodeAt(end + 1) === 0x24) return false;

  const latex = src.slice(pos + 1, end);
  if (!isLikelyInlineLatex(latex)) return false;
  if (!silent) {
    const token = state.push("math_inline", "span", 0);
    token.content = latex;
  }
  state.pos = end + 1;
  return true;
}

function findClosingBlockquote(tokens: StateCore["tokens"], openIndex: number): number {
  let depth = 0;
  for (let i = openIndex; i < tokens.length; i += 1) {
    if (tokens[i]!.type === "blockquote_open") depth += 1;
    else if (tokens[i]!.type === "blockquote_close") {
      depth -= 1;
      if (depth === 0) return i;
    }
  }
  return -1;
}

function stripCalloutPrefix(inlineToken: StateCore["tokens"][number], matchLength: number) {
  inlineToken.content = inlineToken.content.slice(matchLength).trimStart();
  const firstChild = inlineToken.children?.[0];
  if (firstChild?.type === "text") {
    firstChild.content = firstChild.content.replace(CALLOUT_PREFIX, "").trimStart();
  }
}

function convertCalloutAt(tokens: StateCore["tokens"], openIndex: number) {
  const inlineToken = tokens[openIndex + 2];
  if (tokens[openIndex + 1]?.type !== "paragraph_open" || inlineToken?.type !== "inline") return;
  const match = CALLOUT_PREFIX.exec(inlineToken.content);
  const closeIndex = match ? findClosingBlockquote(tokens, openIndex) : -1;
  if (!match || closeIndex === -1) return;
  tokens[openIndex]!.tag = "div";
  tokens[openIndex]!.attrSet("data-type", "callout");
  tokens[openIndex]!.attrSet("data-callout-kind", match[1]!.toLowerCase());
  tokens[closeIndex]!.tag = "div";
  stripCalloutPrefix(inlineToken, match[0].length);
}

function transformCallouts(state: StateCore) {
  for (let i = 0; i < state.tokens.length; i += 1) {
    if (state.tokens[i]!.type === "blockquote_open") convertCalloutAt(state.tokens, i);
  }
}

function convertTaskItem(state: StateCore, listOpenIndex: number, itemOpenIndex: number) {
  const inlineToken = state.tokens[itemOpenIndex + 2];
  if (state.tokens[itemOpenIndex + 1]?.type !== "paragraph_open" || inlineToken?.type !== "inline") return;
  const match = TASK_PREFIX.exec(inlineToken.content);
  if (!match) return;
  const checked = match[1]!.toLowerCase() === "x";
  state.tokens[listOpenIndex]!.attrSet("data-type", "taskList");
  state.tokens[itemOpenIndex]!.attrSet("data-checked", String(checked));
  inlineToken.content = inlineToken.content.slice(match[0].length);
  const first = inlineToken.children?.[0];
  if (first?.type === "text") first.content = first.content.replace(TASK_PREFIX, "");
  const box = new state.Token("task_checkbox", "", 0);
  box.meta = { checked };
  inlineToken.children?.unshift(box);
}

function transformTaskLists(state: StateCore) {
  const listStack: number[] = [];
  for (let i = 0; i < state.tokens.length; i += 1) {
    const type = state.tokens[i]!.type;
    if (type === "bullet_list_open") listStack.push(i);
    else if (type === "bullet_list_close") listStack.pop();
    else if (type === "list_item_open" && listStack.length > 0) {
      convertTaskItem(state, listStack[listStack.length - 1]!, i);
    }
  }
}

/** Registers KaTeX math (`$...$` / `$$...$$`), GFM callout (`> [!NOTE]`), and task lists on markdown-it. */
export function registerRichMarkdownPlugins(md: MarkdownIt): void {
  md.block.ruler.before("fence", "math_block", parseBlockMath, {
    alt: ["paragraph", "reference", "blockquote", "list"],
  });
  md.inline.ruler.before("escape", "math_inline", parseInlineMath);
  md.core.ruler.after("inline", "gfm_callouts", transformCallouts);
  md.core.ruler.after("gfm_callouts", "gfm_task_lists", transformTaskLists);
  registerImageGroupMarkdownPlugin(md);

  md.renderer.rules.task_checkbox = (tokens, index) => {
    const checked = Boolean((tokens[index]?.meta as { checked?: boolean } | undefined)?.checked);
    return `<label class="ui-task-box"><input type="checkbox" disabled${checked ? " checked" : ""} /><span></span></label>`;
  };
  md.renderer.rules.math_block = (tokens, index) => {
    const latex = tokens[index]?.content ?? "";
    return `<div class="ui-math-block" data-type="block-math" data-latex="${escapeAttr(latex)}"></div>\n`;
  };
  md.renderer.rules.math_inline = (tokens, index) => {
    const latex = tokens[index]?.content ?? "";
    return `<span class="ui-math-inline" data-type="inline-math" data-latex="${escapeAttr(latex)}"></span>`;
  };
  const defaultImageRule = md.renderer.rules.image;
  md.renderer.rules.image = (...args) => {
    const [tokens, index, options, env, self] = args;
    const token = tokens[index];
    const src = String(token?.attrGet("src") ?? "");
    const alt = token ? self.renderInlineAsText(token.children ?? [], options, env) : "";
    const mediaHtml = renderMarkdownMediaHtml(src, alt);
    if (mediaHtml) return mediaHtml;
    return defaultImageRule ? defaultImageRule(...args) : self.renderToken(tokens, index, options);
  };
}
