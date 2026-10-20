import type { SpacePlacement } from "../../canvas/navigation-requests";
import type { KnowledgeChunkSeed } from "../../knowledge/model/schema/chunk";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceNodePayload } from "../../model/nodes/node-payload";
import type { SpaceState } from "../session-state";
import type { createSpaceMutations } from "./node-mutations";

type SpaceMutations = ReturnType<typeof createSpaceMutations>;

export interface AddSpaceContentOptions {
  reveal?: boolean;
  sourceChunks?: readonly KnowledgeChunkSeed[];
}

export interface FinalizePendingContentOptions {
  reveal?: boolean;
}

export function createPlacementActions(
  mutations: SpaceMutations,
  placement: SpacePlacement,
  activeSpaceId: SpaceState["activeSpaceId"],
) {
  async function addContent(
    payload: SpaceNodePayload,
    position?: { x: number; y: number },
    options: AddSpaceContentOptions = {},
  ) {
    const spaceId = activeSpaceId.value;
    const contentNodes = await mutations.addContent(payload, position, options.sourceChunks);
    const ids = contentNodes.map((node) => node.id);
    if (activeSpaceId.value === spaceId) {
      if (options.reveal !== false) placement.requestReveal(ids);
    }
    return contentNodes;
  }
  async function addContents(
    contentNodes: SpaceCanvasNode[],
    sourceChunksByNodeId = new Map<string, readonly KnowledgeChunkSeed[]>(),
  ) {
    const spaceId = activeSpaceId.value;
    const addedNodes = await mutations.addContents(contentNodes, sourceChunksByNodeId);
    const ids = addedNodes.map((node) => node.id);
    if (activeSpaceId.value === spaceId) {
      placement.requestReveal(ids);
    }
    return addedNodes;
  }
  function finalizePendingContent(
    spaceId: string,
    contentNodes: readonly SpaceCanvasNode[],
    options: FinalizePendingContentOptions = {},
  ) {
    mutations.commitPendingContent(spaceId, contentNodes);
    if (activeSpaceId.value !== spaceId) return;
    const ids = contentNodes.map((node) => node.id);
    if (options.reveal !== false) placement.requestReveal(ids);
  }
  return { addContent, addContents, finalizePendingContent };
}
