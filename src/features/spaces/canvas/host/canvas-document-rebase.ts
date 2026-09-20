import { isEqual } from "es-toolkit";

import { parseSpaceEditorDocument, type SpaceEditorDocument } from "../contract/canvas-document";
/** Merge disjoint edits; shared-record conflicts fail instead of overwriting topology. */
export function rebaseCanvasDocument(
  base: SpaceEditorDocument,
  committed: SpaceEditorDocument,
  pending: SpaceEditorDocument,
): SpaceEditorDocument {
  return parseSpaceEditorDocument({
    ...committed,
    viewport: isEqual(base.viewport, pending.viewport) ? committed.viewport : pending.viewport,
    structure: {
      version: 1,
      items: merge(
        base.structure.items,
        committed.structure.items,
        pending.structure.items,
        (item) => item.id,
      ),
    },
    layout: {
      version: 1,
      items: merge(base.layout.items, committed.layout.items, pending.layout.items, (item) => item.itemId),
    },
    relationStyles: merge(
      base.relationStyles ?? [],
      committed.relationStyles ?? [],
      pending.relationStyles ?? [],
      (style) => style.relationId,
    ),
  });
}
function merge<T>(
  before: readonly T[],
  committed: readonly T[],
  pending: readonly T[],
  identity: (item: T) => string,
) {
  const base = new Map(before.map((item) => [identity(item), item]));
  const remote = new Map(committed.map((item) => [identity(item), item]));
  const local = new Map(pending.map((item) => [identity(item), item]));
  return [...new Set([...remote.keys(), ...local.keys()])].flatMap((id) => {
    const changed = !isEqual(base.get(id), local.get(id));
    if (changed && !isEqual(base.get(id), remote.get(id)) && !isEqual(local.get(id), remote.get(id)))
      throw new Error(`Concurrent editor change at ${id}`);
    const item = changed ? local.get(id) : remote.get(id);
    return item === undefined ? [] : [item];
  });
}
