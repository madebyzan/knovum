import type { Token } from "markdown-it";

import { createMarkdownRenderer } from "../../../../../shared/ui/markdown-renderer";

const markdown = createMarkdownRenderer();

export interface MarkdownPreviewProjection {
  projectedSource: string;
  remainingBlockCount: number;
  visibleBlockCount: number;
}

export function projectMarkdownPreview(source: string, blockLimit: number): MarkdownPreviewProjection {
  const lines = source.split("\n");
  const blocks = rootBlockRanges(markdown.parse(source, {}));
  const visible = blocks.slice(0, blockLimit);
  if (visible.length === 0) return { projectedSource: "", remainingBlockCount: 0, visibleBlockCount: 0 };
  const start = visible[0]![0];
  const end = visible.at(-1)![1];
  return {
    projectedSource: lines.slice(start, end).join("\n"),
    remainingBlockCount: Math.max(0, blocks.length - visible.length),
    visibleBlockCount: visible.length,
  };
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
