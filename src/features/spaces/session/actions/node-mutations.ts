import { applySpaceNodeContent, createSpaceNode } from "../../canvas/node";
import { refreshEnrichmentJobs } from "../../enrichment/enrichment-jobs-query";
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
import { createNodes, type CreatedSpaceNodes } from "../../nodes/node-operations";
import { spaceEditOperations } from "../../structure/space-edit-operations";
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
    payload: SpaceNodePayload,
    position = context.nextInsertionPoint(),
    sourceChunks: readonly KnowledgeChunkSeed[] = [],
  ) {
    assertSpacePosition(position);
    const node = createSpaceNode(payload, position);
    return addContents([node], new Map([[node.id, sourceChunks]]));
  }

  async function addContents(
    contentNodes: SpaceCanvasNode[],
    sourceChunksByNodeId = new Map<string, readonly KnowledgeChunkSeed[]>(),
  ) {
    const spaceId = requireActiveSpaceId(context.activeSpaceId);
    const targetParentId = currentTargetParentId();
    const releaseMutation = beginSpaceMutation(history);
    try {
      await context.ensureActiveSpacePersisted();
      await persistPendingContent(spaceId, contentNodes, targetParentId, sourceChunksByNodeId);
      commitPendingContent(spaceId, contentNodes);
      return contentNodes;
    } finally {
      releaseMutation();
    }
  }

  async function createPendingContentForSpace(
    spaceId: string,
    payload: SpaceNodePayload,
    position = context.nextInsertionPoint(),
  ) {
    const targetParentId = context.activeSpaceId.value === spaceId ? currentTargetParentId() : null;
    await ensureSpacePersisted(spaceId);
    assertSpacePosition(position);
    const node = createSpaceNode(payload, position);
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
    sourceChunksByNodeId = new Map<string, readonly KnowledgeChunkSeed[]>(),
  ) {
    for (const node of contentNodes) {
      assertSpacePosition(node.initialPosition);
      node.data = validateSpaceNodePayload(node.data);
    }
    const isActive = context.activeSpaceId.value === spaceId;
    if (isActive) nodes.value = [...nodes.value, ...contentNodes];
    let persisted = false;
    try {
      const created = await createNodes({
        spaceId,
        nodes: contentNodes,
        edit: { mutationId: crypto.randomUUID(), targetParentId },
        sourceChunksByNodeId,
      });
      persisted = true;
      await synchronizeCreatedContent(spaceId, contentNodes, created);
    } catch (error) {
      if (!persisted && isActive && context.activeSpaceId.value === spaceId) {
        const pendingIds = new Set(contentNodes.map(({ id }) => id));
        nodes.value = nodes.value.filter(({ id }) => !pendingIds.has(id));
      }
      throw error;
    }
  }

  /**
   * A creation answers with its nodes as stored, so only the Space's geometry is read back. Every
   * other node is already in the session; reloading all of them made a new note wait on every
   * payload the Space holds before it could appear.
   */
  async function synchronizeCreatedContent(
    spaceId: string,
    contentNodes: SpaceCanvasNode[],
    created: CreatedSpaceNodes,
  ) {
    for (const pending of contentNodes) {
      const committed = created.payloadById.get(pending.id);
      if (!committed) throw new Error("Created node is missing from its committed creation");
      pending.data = committed;
    }
    if (context.activeSpaceId.value !== spaceId) return;
    context.nodes.value = context.nodes.value.map((node) => {
      const committed = created.payloadById.get(node.id);
      return committed ? applySpaceNodeContent(node, committed) : node;
    });
    // A snapshot at or past this revision already contains the creation. Parallel creations read
    // their snapshot after their own commit, so a slower read can also return an older Space than
    // the session holds; only newer snapshots win.
    if (context.editSnapshot.value.editRevision >= created.editRevision) return;
    const editSnapshot = await spaceEditOperations.load(spaceId);
    if (context.activeSpaceId.value !== spaceId) return;
    if (editSnapshot.editRevision < context.editSnapshot.value.editRevision) return;
    context.editSnapshot.value = editSnapshot;
  }

  function commitPendingContent(spaceId: string, contentNodes: readonly SpaceCanvasNode[]) {
    if (context.activeSpaceId.value === spaceId) {
      context.recordCommittedSpaceEdit();
    }
    scheduleKnowledgeIndex(
      spaceId,
      contentNodes.map((node) => node.id),
    );
    void refreshEnrichmentJobs();
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
