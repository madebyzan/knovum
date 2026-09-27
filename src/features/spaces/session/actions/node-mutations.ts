import { createSpaceNode } from "../../canvas/node";
import { refreshEnrichmentTasks } from "../../enrichment/job-observation";
import { createSpaceHistoryMutations } from "../../history/mutation-actions";
import { beginSpaceMutation } from "../../history/mutation-barrier";
import {
  assertSpacePosition,
  requireActiveSpaceId,
  type SpaceMutationContext,
} from "../../history/mutation-context";
import { scheduleKnowledgeIndex } from "../../knowledge/indexing/index-scheduler";
import type { KnowledgeChunkSeed } from "../../knowledge/model/schema/chunk";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceNodePayload } from "../../model/nodes/node-payload";
import { validateSpaceNodePayload } from "../../model/nodes/node-validation";
import { createNodes } from "../../nodes/node-operations";
import { spaceEditOperations } from "../../structure/space-edit-operations";
import { loadSpaceSessionProjection } from "../space-projection";
import { createNodeContentMutations } from "./node-content-mutations";

export function createSpaceMutations(context: SpaceMutationContext) {
  const dataMutations = createNodeContentMutations(context);
  return {
    ...createContentMutations(context),
    ...dataMutations,
    ...createSpaceHistoryMutations(context),
  };
}

function createContentMutations(context: SpaceMutationContext) {
  const { nodes, history } = context;

  async function addContent(
    data: SpaceNodePayload,
    position = context.nextInsertionPoint(),
    sourceChunks: readonly KnowledgeChunkSeed[] = [],
  ) {
    assertSpacePosition(position);
    const node = createSpaceNode(data, position);
    return addContents([node], new Map([[node.id, sourceChunks]]));
  }

  async function addContents(
    contentNodes: SpaceCanvasNode[],
    sourceChunks = new Map<string, readonly KnowledgeChunkSeed[]>(),
  ) {
    const spaceId = requireActiveSpaceId(context.activeSpaceId);
    const targetParentId = currentTargetParentId();
    const releaseMutation = beginSpaceMutation(history);
    try {
      await context.ensureActiveSpacePersisted();
      await persistPendingContent(spaceId, contentNodes, targetParentId, sourceChunks);
      commitPendingContent(spaceId, contentNodes);
      return contentNodes;
    } finally {
      releaseMutation();
    }
  }

  async function createPendingContentForSpace(
    spaceId: string,
    data: SpaceNodePayload,
    position = context.nextInsertionPoint(),
  ) {
    const targetParentId = context.activeSpaceId.value === spaceId ? currentTargetParentId() : null;
    await ensureSpacePersisted(spaceId);
    assertSpacePosition(position);
    const node = createSpaceNode(data, position);
    await persistPendingContent(spaceId, [node], targetParentId);
    return [node];
  }

  async function ensureSpacePersisted(spaceId: string) {
    if (context.activeSpaceId.value !== spaceId) return;
    await context.ensureActiveSpacePersisted();
  }

  async function persistPendingContent(
    spaceId: string,
    contentNodes: SpaceCanvasNode[],
    targetParentId: string | null,
    sourceChunks = new Map<string, readonly KnowledgeChunkSeed[]>(),
  ) {
    for (const node of contentNodes) {
      assertSpacePosition(node.initialPosition);
      node.data = validateSpaceNodePayload(node.data);
    }
    const isActive = context.activeSpaceId.value === spaceId;
    if (isActive) nodes.value = [...nodes.value, ...contentNodes];
    let persisted = false;
    try {
      const snapshot = isActive ? context.editSnapshot.value : await spaceEditOperations.load(spaceId);
      await createNodes(
        spaceId,
        contentNodes,
        {
          mutationId: crypto.randomUUID(),
          expectedEditRevision: snapshot.editRevision,
          targetParentId,
        },
        sourceChunks,
      );
      persisted = true;
      await synchronizeCreatedContent(spaceId, contentNodes);
    } catch (error) {
      if (!persisted && isActive && context.activeSpaceId.value === spaceId) {
        const pendingIds = new Set(contentNodes.map(({ id }) => id));
        nodes.value = nodes.value.filter(({ id }) => !pendingIds.has(id));
      }
      throw error;
    }
  }

  async function synchronizeCreatedContent(spaceId: string, contentNodes: SpaceCanvasNode[]) {
    const projection = await loadSpaceSessionProjection(spaceId);
    for (const pending of contentNodes) {
      const committed = projection.nodes.find(({ id }) => id === pending.id);
      if (!committed) throw new Error("Created node projection is unavailable");
      pending.data = committed.data;
    }
    if (context.activeSpaceId.value === spaceId) {
      context.nodes.value = projection.nodes;
      context.editSnapshot.value = projection.editSnapshot;
    }
  }

  function commitPendingContent(
    spaceId: string,
    contentNodes: readonly SpaceCanvasNode[],
    historyMode: "record" | "return" | "none" = "record",
  ) {
    if (context.activeSpaceId.value === spaceId && historyMode === "record") {
      context.recordCommittedSpaceEdit();
    }
    scheduleKnowledgeIndex(
      spaceId,
      contentNodes.map((node) => node.id),
    );
    void refreshEnrichmentTasks(spaceId);
  }

  function currentTargetParentId() {
    return context.groupScopePath.value.at(-1) ?? null;
  }

  return {
    addContent,
    addContents,
    createPendingContentForSpace,
    commitPendingContent,
  };
}
