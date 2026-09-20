import { createSpaceNode, snapshotSpaceNode } from "../canvas/node";
import {
  resumePendingEnrichmentJobs,
  type SpaceEnrichmentSchedulerContext,
} from "../enrichment/job-scheduler";
import { createSpaceHistoryMutations } from "../history/mutation-actions";
import { createAddHistory, createDeleteHistory } from "../history/mutation-application";
import { beginSpaceMutation } from "../history/mutation-barrier";
import {
  assertSpacePosition,
  requireActiveSpaceId,
  type SpaceMutationContext,
} from "../history/mutation-types";
import { recordSpaceHistory } from "../history/stack";
import { scheduleKnowledgeIndex } from "../knowledge/indexing/index-scheduler";
import { prepareSpaceKnowledgePayload } from "../knowledge/model/node-knowledge";
import type { KnowledgeChunkSeed } from "../knowledge/model/schema";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceNodePayload } from "../model/nodes/node-payload";
import { validateSpaceNodePayload } from "../model/nodes/schema/node-payload";
import { createNodes, setNodesDeleted } from "./node-repository";
import { createNodeDataMutations } from "./node-data-mutations";

export type { NodeDataChange } from "../history/mutation-types";

export function createSpaceMutations(context: SpaceMutationContext) {
  const dataMutations = createNodeDataMutations(context);
  return {
    ...createItemMutations(context, { applyNodeDataForSpace: dataMutations.applyNodeDataForSpace }),
    ...dataMutations,
    ...createSpaceHistoryMutations(context),
  };
}

function createItemMutations(context: SpaceMutationContext, enrichment: SpaceEnrichmentSchedulerContext) {
  const { nodes, history } = context;

  async function addItem(
    data: SpaceNodePayload,
    position = context.nextInsertionPoint(),
    sourceChunks: readonly KnowledgeChunkSeed[] = [],
  ) {
    assertSpacePosition(position);
    const node = createSpaceNode(data, position);
    return addItems([node], new Map([[node.id, sourceChunks]]));
  }

  async function addItems(
    items: SpaceCanvasNode[],
    sourceChunks = new Map<string, readonly KnowledgeChunkSeed[]>(),
  ) {
    const spaceId = requireActiveSpaceId(context.activeSpaceId);
    const releaseMutation = beginSpaceMutation(history);
    try {
      await context.ensureActiveSpacePersisted();
      await persistPendingItems(spaceId, items, sourceChunks);
      commitPendingItems(spaceId, items);
      return items;
    } finally {
      releaseMutation();
    }
  }

  async function createPendingItemForSpace(
    spaceId: string,
    data: SpaceNodePayload,
    position = context.nextInsertionPoint(),
  ) {
    await ensureSpacePersisted(spaceId);
    assertSpacePosition(position);
    const node = createSpaceNode(data, position);
    await persistPendingItems(spaceId, [node]);
    return [node];
  }

  async function ensureSpacePersisted(spaceId: string) {
    if (context.activeSpaceId.value !== spaceId) return;
    await context.ensureActiveSpacePersisted();
  }

  async function persistPendingItems(
    spaceId: string,
    items: SpaceCanvasNode[],
    sourceChunks = new Map<string, readonly KnowledgeChunkSeed[]>(),
  ) {
    for (const item of items) {
      assertSpacePosition(item.initialPosition);
      item.data = prepareSpaceKnowledgePayload(validateSpaceNodePayload(item.data));
    }
    const isActive = context.activeSpaceId.value === spaceId;
    if (isActive) nodes.value = [...nodes.value, ...items];
    try {
      await createNodes(spaceId, items, sourceChunks);
    } catch (error) {
      if (isActive && context.activeSpaceId.value === spaceId) {
        const pendingIds = new Set(items.map(({ id }) => id));
        nodes.value = nodes.value.filter(({ id }) => !pendingIds.has(id));
      }
      throw error;
    }
  }

  function commitPendingItems(
    spaceId: string,
    items: readonly SpaceCanvasNode[],
    historyMode: "record" | "return" | "none" = "record",
  ) {
    const entry = createAddHistory(spaceId, items, nodes, context.activeSpaceId);
    if (context.activeSpaceId.value === spaceId) {
      if (historyMode === "record") recordSpaceHistory(history, entry);
    }
    scheduleKnowledgeIndex(
      spaceId,
      items.map((item) => item.id),
    );
    void resumePendingEnrichmentJobs(enrichment, spaceId);
    return historyMode === "return" ? entry : undefined;
  }

  async function deleteItems(ids: readonly string[]) {
    const spaceId = requireActiveSpaceId(context.activeSpaceId);
    const idSet = new Set(ids);
    const removed = nodes.value.filter((node) => idSet.has(node.id)).map(snapshotSpaceNode);
    if (removed.length === 0) return;
    const releaseMutation = beginSpaceMutation(history);
    try {
      await setNodesDeleted(
        spaceId,
        removed.map((node) => node.id),
        true,
      );
      if (context.activeSpaceId.value === spaceId)
        nodes.value = nodes.value.filter((node) => !idSet.has(node.id));
      recordSpaceHistory(history, createDeleteHistory(spaceId, removed, nodes, context.activeSpaceId));
    } finally {
      releaseMutation();
    }
  }

  return {
    addItem,
    addItems,
    createPendingItemForSpace,
    commitPendingItems,
    deleteItems,
  };
}
