import { countBy, take } from "es-toolkit";
import { search } from "fast-fuzzy";

import type { SpaceCanvasNode } from "../model/nodes/canvas-node";

export function selectedTagCounts(nodes: readonly SpaceCanvasNode[]) {
  return new Map(
    Object.entries(
      countBy(
        nodes.flatMap((node) => node.data.tags ?? []),
        (tag) => tag,
      ),
    ),
  );
}

export function spaceTagCounts(nodes: readonly SpaceCanvasNode[]) {
  return Object.entries(
    countBy(
      nodes.flatMap((node) => node.data.tags ?? []),
      (tag) => tag,
    ),
  ).toSorted(([left], [right]) => left.localeCompare(right));
}

export function suggestTags(
  query: string,
  counts: readonly [string, number][],
  selected: ReadonlyMap<string, number>,
) {
  if (!query.trim()) return [];
  const candidates = counts.filter(([tag]) => !selected.has(tag));
  return take(search(query.trim(), candidates, { keySelector: ([tag]) => tag }), 5);
}
