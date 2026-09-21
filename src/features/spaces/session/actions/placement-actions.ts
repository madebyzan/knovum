import type { SpacePlacement } from "../../canvas/navigation-requests";
import type { KnowledgeChunkSeed } from "../../knowledge/model/schema";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceNodePayload } from "../../model/nodes/node-payload";
import type { SpaceState } from "../session-state";
import type { createSpaceMutations } from "./node-mutations";

type SpaceMutations = ReturnType<typeof createSpaceMutations>;

export interface AddSpaceContentOptions {
  reveal?: boolean;
  sourceChunks?: readonly KnowledgeChunkSeed[];
}

export function createPlacementActions(
  mutations: SpaceMutations,
  placement: SpacePlacement,
  activeSpaceId: SpaceState["activeSpaceId"],
) {
  async function addContent(
    data: SpaceNodePayload,
    position?: { x: number; y: number },
    options: AddSpaceContentOptions = {},
  ) {
    const spaceId = activeSpaceId.value;
    const contentNodes = await mutations.addContent(data, position, options.sourceChunks);
    const ids = contentNodes.map((node) => node.id);
    if (activeSpaceId.value === spaceId) {
      if (options.reveal !== false) placement.requestReveal(ids);
    }
    return contentNodes;
  }
  async function addContents(
    contentNodes: SpaceCanvasNode[],
    sourceChunks = new Map<string, readonly KnowledgeChunkSeed[]>(),
  ) {
    const spaceId = activeSpaceId.value;
    const addedNodes = await mutations.addContents(contentNodes, sourceChunks);
    const ids = addedNodes.map((node) => node.id);
    if (activeSpaceId.value === spaceId) {
      placement.requestReveal(ids);
    }
    return addedNodes;
  }
  function finalizePendingContent(spaceId: string, contentNodes: readonly SpaceCanvasNode[], reveal = true) {
    mutations.commitPendingContent(spaceId, contentNodes);
    if (activeSpaceId.value !== spaceId) return;
    const ids = contentNodes.map((node) => node.id);
    if (reveal) placement.requestReveal(ids);
  }
  return { addContent, addContents, finalizePendingContent };
}
