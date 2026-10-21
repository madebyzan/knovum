import DOMPurify from "dompurify";
import type { Env, MarkdownIt, Token } from "markdown-it";

import type { MarkdownCitations } from "./markdown-renderer";

/**
 * What a source renders to, split where it can no longer change: `settled` holds sanitized HTML
 * for every top-level block that a later block has closed, in order and append-only; `open` is the
 * last block, which further text may still extend or reshape. `restarted` reports that markup
 * painted from earlier frames no longer holds and must be discarded.
 */
export interface MarkdownFrame {
  settled: readonly string[];
  open: string;
  restarted: boolean;
}

type References = NonNullable<Env["references"]>;

interface StreamEnvironment extends Env {
  citations?: MarkdownCitations;
  references: References;
}

/**
 * Renders a source that grows by appending, such as an answer being streamed. A top-level block
 * followed by another block is final, so each call parses only the text from the start of the
 * last open block: the cost of a frame follows the length of that block, not of the answer.
 *
 * The one construct that reaches backwards is a reference definition — `[label]: url` makes
 * `[text][label]` written earlier a link. The stream therefore tracks every definition in the
 * source, and when one appears or changes it renders the whole source again, so a streamed answer
 * stays identical to rendering its final text at once.
 */
export function createMarkdownStream(md: MarkdownIt, citations?: MarkdownCitations) {
  let settledSource = "";
  let settled: string[] = [];
  // Definitions from settled text only: markdown-it keeps the first definition of a label, so a
  // definition still being written in the open block must not be frozen into later frames.
  let settledReferences: References = {};
  let definitions = "";

  const environment = (): StreamEnvironment => ({
    ...(citations ? { citations } : {}),
    references: { ...settledReferences },
  });

  function reset() {
    settledSource = "";
    settled = [];
    settledReferences = {};
  }

  function settle(tokens: Token[], pending: string, frame: StreamEnvironment): number {
    const open = lastBlockStart(tokens);
    if (open === 0) return 0;
    const source = pending.slice(0, lineOffset(pending, tokens[open]!.map![0]));
    settled.push(DOMPurify.sanitize(md.renderer.render(tokens.slice(0, open), md.options, frame)));
    const own = environment();
    md.parse(source, own);
    settledReferences = own.references;
    settledSource += source;
    return open;
  }

  return (source: string): MarkdownFrame => {
    let restarted = !source.startsWith(settledSource);
    if (restarted) reset();
    let frame = environment();
    let pending = source.slice(settledSource.length);
    let tokens = md.parse(pending, frame);
    if (settled.length && JSON.stringify(frame.references) !== definitions) {
      reset();
      restarted = true;
      frame = environment();
      pending = source;
      tokens = md.parse(pending, frame);
    }
    definitions = JSON.stringify(frame.references);
    const open = settle(tokens, pending, frame);
    const openHtml = md.renderer.render(tokens.slice(open), md.options, frame);
    return { settled, open: DOMPurify.sanitize(openHtml), restarted };
  };
}

/** Index of the token opening the last top-level block, or 0 when there is at most one. */
function lastBlockStart(tokens: readonly Token[]): number {
  let start = 0;
  tokens.forEach((token, index) => {
    if (token.level === 0 && token.nesting !== -1 && token.map) start = index;
  });
  return start;
}

function lineOffset(text: string, line: number): number {
  let offset = 0;
  for (let current = 0; current < line; current += 1) offset = text.indexOf("\n", offset) + 1;
  return offset;
}
