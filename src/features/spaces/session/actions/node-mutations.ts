import { createSpaceNode } from "../../canvas/node";
import {
  resumePendingEnrichmentJobs,
  type SpaceEnrichmentSchedulerContext,
} from "../../enrichment/job-scheduler";
import { createSpaceHistoryMutations } from "../../history/mutation-actions";
import { beginSpaceMutation } from "../../history/mutation-barrier";
import {
  assertSpacePosition,
  requireActiveSpaceId,
  type SpaceMutationContext,
} from "../../history/mutation-types";
import { scheduleKnowledgeIndex } from "../../knowledge/indexing/index-scheduler";
import { prepareSpaceKnowledgePayload } from "../../knowledge/model/node-knowledge";
import type { KnowledgeChunkSeed } from "../../knowledge/model/schema";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceNodePayload } from "../../model/nodes/node-payload";
import { validateSpaceNodePayload } from "../../model/nodes/schema/node-payload";
import { createNodes } from "../../nodes/node-repository";
import { spaceEditRepository } from "../../structure/space-edit-repository";
import { createNodeDataMutations } from "./node-data-mutations";

export type { NodeDataChange } from "../../history/mutation-types";

export function createSpaceMutations(context: SpaceMutationContext) {
  const dataMutations = createNodeDataMutations(context);
  return {
    ...createContentMutations(context, { applyNodeDataForSpace: dataMutations.applyNodeDataForSpace }),
    ...dataMutations,
    ...createSpaceHistoryMutations(context),
  };
}

function createContentMutations(context: SpaceMutationContext, enrichment: SpaceEnrichmentSchedulerContext) {
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
      node.data = prepareSpaceKnowledgePayload(validateSpaceNodePayload(node.data));
    }
    const isActive = context.activeSpaceId.value === spaceId;
    if (isActive) nodes.value = [...nodes.value, ...contentNodes];
    let persisted = false;
    try {
      const snapshot = isActive ? context.editSnapshot.value : await spaceEditRepository.load(spaceId);
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
      const committedSnapshot = await spaceEditRepository.load(spaceId);
      if (context.activeSpaceId.value === spaceId) context.editSnapshot.value = committedSnapshot;
    } catch (error) {
      if (!persisted && isActive && context.activeSpaceId.value === spaceId) {
        const pendingIds = new Set(contentNodes.map(({ id }) => id));
        nodes.value = nodes.value.filter(({ id }) => !pendingIds.has(id));
      }
      throw error;
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
    void resumePendingEnrichmentJobs(enrichment, spaceId);
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
