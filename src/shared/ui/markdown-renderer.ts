import MarkdownIt, { type MarkdownIt as MarkdownItInstance } from "markdown-it";

/**
 * Citation markers an answer may carry, keyed by index: `true` when the cited
 * passage can still be opened. Supplied per render through the markdown-it
 * environment, so one renderer serves every message.
 */
export type MarkdownCitations = ReadonlyMap<number, boolean>;

type CitationEnvironment = { citations?: MarkdownCitations };

const CITATION_MARKER = /^\[C([1-9]\d*)\]/u;

interface CitationMeta {
  index: number;
  openable: boolean;
}

function citationHtml(meta: CitationMeta | undefined): string {
  if (!meta) throw new Error("Citation token carries no citation");
  return meta.openable
    ? `<button type="button" class="ui-citation" data-citation-index="${meta.index}">${meta.index}</button>`
    : `<span class="ui-citation" data-citation-detached="">${meta.index}</span>`;
}

export function createMarkdownRenderer(): MarkdownItInstance {
  const md = new MarkdownIt({
    html: false,
    linkify: true,
    breaks: false,
    typographer: true,
  });
  md.inline.ruler.before("link", "citation", (state, silent) => {
    const citations = (state.env as CitationEnvironment | undefined)?.citations;
    if (!citations || state.src.charCodeAt(state.pos) !== 0x5b) return false;
    const match = CITATION_MARKER.exec(state.src.slice(state.pos));
    const index = match ? Number(match[1]) : Number.NaN;
    const openable = citations.get(index);
    if (!match || openable === undefined) return false;
    if (!silent) {
      const token = state.push("citation", "", 0);
      token.meta = { index, openable } satisfies CitationMeta;
    }
    state.pos += match[0].length;
    return true;
  });
  md.renderer.rules.citation = (tokens, index) =>
    citationHtml(tokens[index]?.meta as CitationMeta | undefined);
  return md;
}
