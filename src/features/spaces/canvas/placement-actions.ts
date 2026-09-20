import type { SpacePlacement } from "./navigation-requests";
import type { KnowledgeChunkSeed } from "../knowledge/model/schema";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceNodePayload } from "../model/nodes/node-payload";
import type { SpaceState } from "../session/session-state";
import type { createSpaceMutations } from "../nodes/node-mutations";

type SpaceMutations = ReturnType<typeof createSpaceMutations>;

export interface AddSpaceItemOptions {
  reveal?: boolean;
  sourceChunks?: readonly KnowledgeChunkSeed[];
}

export function createPlacementActions(
  mutations: SpaceMutations,
  placement: SpacePlacement,
  activeSpaceId: SpaceState["activeSpaceId"],
) {
  async function addItem(
    data: SpaceNodePayload,
    position?: { x: number; y: number },
    options: AddSpaceItemOptions = {},
  ) {
    const spaceId = activeSpaceId.value;
    const items = await mutations.addItem(data, position, options.sourceChunks);
    const ids = items.map((item) => item.id);
    if (activeSpaceId.value === spaceId) {
      if (options.reveal !== false) placement.requestReveal(ids);
    }
    return items;
  }
  async function addItems(
    items: SpaceCanvasNode[],
    sourceChunks = new Map<string, readonly KnowledgeChunkSeed[]>(),
  ) {
    const spaceId = activeSpaceId.value;
    const added = await mutations.addItems(items, sourceChunks);
    const ids = added.map((item) => item.id);
    if (activeSpaceId.value === spaceId) {
      placement.requestReveal(ids);
    }
    return added;
  }
  function finalizePendingItems(spaceId: string, items: readonly SpaceCanvasNode[], reveal = true) {
    mutations.commitPendingItems(spaceId, items);
    if (activeSpaceId.value !== spaceId) return;
    const ids = items.map((item) => item.id);
    if (reveal) placement.requestReveal(ids);
  }
  return { addItem, addItems, finalizePendingItems };
}
