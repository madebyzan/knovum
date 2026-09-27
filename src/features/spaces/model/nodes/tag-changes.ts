import { isEqual } from "es-toolkit";

import type { NodePayloadChange } from "../../history/mutation-types";
import type { SpaceCanvasNode } from "./canvas-node";
import type { SpaceNodePayload } from "./node-payload";

export function createTagChanges(
  nodes: readonly SpaceCanvasNode[],
  update: (tags: readonly string[]) => string[],
): NodePayloadChange[] {
  return nodes.flatMap((node) => {
    const current = node.data.tags ?? [];
    const next = update(current);
    return isEqual(current, next)
      ? []
      : [{ id: node.id, before: node.data, after: dataWithTags(node.data, next) }];
  });
}

function dataWithTags(data: SpaceNodePayload, tags: readonly string[]): SpaceNodePayload {
  const { tags: _currentTags, ...untagged } = data;
  // Tags are part of every payload variant's shared base contract.
  return (tags.length ? { ...untagged, tags: [...tags] } : untagged) as SpaceNodePayload;
}
