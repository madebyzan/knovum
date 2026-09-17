import { omit as esOmit } from "es-toolkit";
import type { Ref } from "vue";

import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceNodePayload } from "../model/nodes/node-payload";
import { validateSpaceNodePayload } from "../model/nodes/schema/node-payload";
import type { NodeDataChange } from "../session/actions/node-mutations";

export interface SelectionActionContext {
  nodes: Ref<SpaceCanvasNode[]>;
  addItem: (
    data: SpaceNodePayload,
    position?: { x: number; y: number },
    options?: { reveal?: boolean },
  ) => Promise<SpaceCanvasNode[]>;
  deleteItems: (ids: readonly string[]) => Promise<void>;
  removeRelations: (ids: readonly string[]) => Promise<void>;
  updateManyData: (changes: readonly NodeDataChange[]) => Promise<void>;
  recordHistoryMutation: (action: () => Promise<void>) => Promise<void>;
}

export async function updateSelected(
  context: SelectionActionContext,
  ids: readonly string[],
  mutate: (data: SpaceNodePayload) => SpaceNodePayload,
  options: { includeSections?: boolean } = {},
) {
  const selected = new Set(ids);
  const changes = context.nodes.value.flatMap<NodeDataChange>((node) => {
    if (!selected.has(node.id) || (!options.includeSections && node.data.kind === "section")) return [];
    const before = validateSpaceNodePayload(node.data);
    return [{ id: node.id, before, after: mutate(validateSpaceNodePayload(node.data)) }];
  });
  await context.updateManyData(changes);
}

export function omit(
  data: SpaceNodePayload,
  key: "sectionId" | "tags" | "locked" | "collapsed",
): SpaceNodePayload {
  return esOmit(data as unknown as Record<string, unknown>, [key]) as SpaceNodePayload;
}
