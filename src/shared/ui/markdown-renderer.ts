import MarkdownIt, { type MarkdownIt as MarkdownItInstance, type StateCore, type Token } from "markdown-it";

import { linkDestination } from "../contracts/link-destination";

/**
 * Citation markers an answer or grounded document may carry, keyed by index: `true` when the
 * cited passage can still be opened. `"preview"` renders any `[Cn]` marker as a non-interactive
 * superscript badge for glanceable surfaces (such as canvas cards) without altering text.
 */
export type MarkdownCitations = ReadonlyMap<number, boolean> | "preview";

type CitationEnvironment = { citations?: MarkdownCitations };

const CITATION_MARKER = /^\[C([1-9]\d*)\]/u;

interface CitationMeta {
  index: number;
  mode: "openable" | "detached" | "preview";
}

function citationHtml(meta: CitationMeta | undefined): string {
  if (!meta) throw new Error("Citation token carries no citation");
  if (meta.mode === "openable")
    return `<button type="button" class="ui-citation" data-citation-index="${meta.index}">${meta.index}</button>`;
  if (meta.mode === "preview")
    return `<span class="ui-citation" data-citation-index="${meta.index}" data-citation-preview="">${meta.index}</span>`;
  return `<span class="ui-citation" data-citation-index="${meta.index}" data-citation-detached="">${meta.index}</span>`;
}

export interface MarkdownRendererOptions {
  /** Keeps links to local files, for a surface that can follow them. */
  localLinks?: boolean;
}

/**
 * A link is rendered only where following it leads somewhere; any other keeps its label as text.
 * Markdown links do not nest, so one flag tracks the link being read.
 */
function withoutDeadLinks(tokens: Token[], localLinks: boolean): Token[] {
  let dead = false;
  return tokens.filter((token) => {
    if (token.type === "link_open") {
      const destination = linkDestination(String(token.attrGet("href") ?? ""));
      dead = !(destination === "external" || (destination === "local" && localLinks));
    } else if (token.type !== "link_close") return true;
    const kept = !dead;
    if (token.type === "link_close") dead = false;
    return kept;
  });
}

export function createMarkdownRenderer({
  localLinks = false,
}: MarkdownRendererOptions = {}): MarkdownItInstance {
  const md = new MarkdownIt({
    html: false,
    linkify: true,
    breaks: false,
    typographer: true,
  });
  // Only text that spells out its scheme is a link: a fuzzy domain match turns file names into
  // hosts — `README.md` is in Moldova, `main.rs` in Serbia.
  md.linkify.set({ fuzzyLink: false });
  md.core.ruler.push("unlink_dead_destinations", (state: StateCore) => {
    for (const block of state.tokens) {
      if (block.children) block.children = withoutDeadLinks(block.children, localLinks);
    }
  });
  md.inline.ruler.before("link", "citation", (state, silent) => {
    const citations = (state.env as CitationEnvironment | undefined)?.citations;
    if (!citations || state.src.charCodeAt(state.pos) !== 0x5b) return false;
    const match = CITATION_MARKER.exec(state.src.slice(state.pos));
    const index = match ? Number(match[1]) : Number.NaN;
    if (!match) return false;
    const mode = resolveCitationMode(citations, index);
    if (!mode) return false;
    if (!silent) {
      const token = state.push("citation", "", 0);
      token.meta = { index, mode } satisfies CitationMeta;
    }
    state.pos += match[0].length;
    return true;
  });
  md.renderer.rules.citation = (tokens, index) =>
    citationHtml(tokens[index]?.meta as CitationMeta | undefined);
  return md;
}

function resolveCitationMode(
  citations: MarkdownCitations,
  index: number,
): CitationMeta["mode"] | undefined {
  if (citations === "preview") return "preview";
  const openable = citations.get(index);
  if (openable === undefined) return undefined;
  return openable ? "openable" : "detached";
}
