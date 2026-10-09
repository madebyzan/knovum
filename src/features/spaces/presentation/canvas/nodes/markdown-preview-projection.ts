import type { Token } from "markdown-it";

import { createMarkdownRenderer } from "../../../../../shared/ui/markdown-renderer";
import { CANVAS_PREVIEW_CHARACTERS_PER_ITEM } from "../../../canvas/preview/policy";

const markdown = createMarkdownRenderer();
const parsedMarkdownBySource = new WeakMap<
  { text: string },
  {
    truncated: boolean;
    lines: string[];
    blocks: [number, number][];
    previews: Map<number, MarkdownPreviewProjection>;
  }
>();

export interface MarkdownPreviewProjection {
  projectedSource: string;
  remainingBlockCount: number;
  visibleBlockCount: number;
  truncated: boolean;
}

export function projectMarkdownPreview(
  payload: { text: string },
  blockLimit: number,
): MarkdownPreviewProjection {
  let document = parsedMarkdownBySource.get(payload);
  if (!document) {
    const source = payload.text.slice(0, 10 * CANVAS_PREVIEW_CHARACTERS_PER_ITEM);
    document = {
      truncated: source.length < payload.text.length,
      lines: source.split("\n"),
      blocks: rootBlockRanges(markdown.parse(source, {})),
      previews: new Map(),
    };
    parsedMarkdownBySource.set(payload, document);
  }
  const cached = document.previews.get(blockLimit);
  if (cached) return cached;
  const visible = document.blocks.slice(0, blockLimit);
  const source = visible.length ? document.lines.slice(visible[0]![0], visible.at(-1)![1]).join("\n") : "";
  const characterLimit = blockLimit * CANVAS_PREVIEW_CHARACTERS_PER_ITEM;
  const preview = {
    projectedSource: source.slice(0, characterLimit),
    remainingBlockCount: document.truncated ? 0 : Math.max(0, document.blocks.length - visible.length),
    visibleBlockCount: visible.length,
    truncated: document.truncated || source.length > characterLimit,
  };
  document.previews.set(blockLimit, preview);
  return preview;
}

function rootBlockRanges(tokens: readonly Token[]): [number, number][] {
  const ranges: [number, number][] = [];
  for (const token of tokens) {
    if (token.level !== 0 || token.nesting === -1 || !token.map) continue;
    const range: [number, number] = [token.map[0], token.map[1]];
    const previous = ranges.at(-1);
    if (!previous || previous[0] !== range[0] || previous[1] !== range[1]) ranges.push(range);
  }
  return ranges;
}
