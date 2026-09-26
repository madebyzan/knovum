import { cleanupDeletedSpaceAssets } from "../../assets/asset-operations";
import { applySpaceNodeData } from "../../canvas/node";
import { resumePendingEnrichmentJobs } from "../../enrichment/job-scheduler";
import { beginSpaceMutation } from "../../history/mutation-barrier";
import { requireActiveSpaceId, type SpaceMutationContext } from "../../history/mutation-types";
import { scheduleKnowledgeIndex } from "../../knowledge/indexing/index-scheduler";
import { prepareSpaceKnowledgePayload } from "../../knowledge/model/node-knowledge";
import type { KnowledgeChunkSeed } from "../../knowledge/model/schema";
import type { SpaceNodePayload } from "../../model/nodes/node-payload";
import { validateSpaceNodePayload } from "../../model/nodes/schema/node-payload";
import {
  discardPendingContent as persistPendingContentDiscard,
  updateNodePayload,
} from "../../nodes/node-operations";
import { spaceEditOperations } from "../../structure/space-edit-operations";
import { loadSpaceSessionProjection } from "../space-projection";

export function createNodeDataMutations(context: SpaceMutationContext) {
  async function saveNodeData(
    id: string,
    expectedData: SpaceNodePayload,
    data: SpaceNodePayload,
    sourceChunks?: readonly KnowledgeChunkSeed[],
  ) {
    const releaseMutation = beginSpaceMutation(context.history);
    try {
      const spaceId = requireActiveSpaceId(context.activeSpaceId);
      await persistData(spaceId, { id, expectedData, data, sourceChunks });
      afterSaved(spaceId, id);
    } finally {
      releaseMutation();
    }
  }

  async function saveNodeDataForSpace(
    spaceId: string,
    id: string,
    expectedData: SpaceNodePayload,
    data: SpaceNodePayload,
  ) {
    await persistData(spaceId, { id, expectedData, data });
    afterSaved(spaceId, id);
  }

  function afterSaved(spaceId: string, id: string) {
    scheduleKnowledgeIndex(spaceId, [id]);
    void resumePendingEnrichmentJobs({ applyNodeDataForSpace }, spaceId);
  }

  async function savePendingNodeDataForSpace(
    spaceId: string,
    id: string,
    expectedData: SpaceNodePayload,
    data: SpaceNodePayload,
  ) {
    await persistData(spaceId, { id, expectedData, data });
  }

  async function persistData(
    spaceId: string,
    change: {
      id: string;
      expectedData: SpaceNodePayload;
      data: SpaceNodePayload;
      sourceChunks?: readonly KnowledgeChunkSeed[] | undefined;
    },
  ) {
    const validated = prepareSpaceKnowledgePayload(validateSpaceNodePayload(change.data));
    await updateNodePayload(spaceId, { ...change, data: validated });
    patchActiveNodeData(spaceId, change.id, validated);
  }

  function applyNodeDataForSpace(spaceId: string, id: string, data: SpaceNodePayload) {
    const validated = validateSpaceNodePayload(data);
    patchActiveNodeData(spaceId, id, validated);
    scheduleKnowledgeIndex(spaceId, [id]);
  }

  async function discardPendingContent(spaceId: string, id: string) {
    const snapshot =
      context.activeSpaceId.value === spaceId
        ? context.editSnapshot.value
        : await spaceEditOperations.load(spaceId);
    const result = await persistPendingContentDiscard({
      spaceId,
      nodeId: id,
      expectedEditRevision: snapshot.editRevision,
      mutationId: crypto.randomUUID(),
    });
    if (context.activeSpaceId.value === spaceId) {
      const projection = await loadSpaceSessionProjection(spaceId);
      if (context.activeSpaceId.value === spaceId) {
        context.nodes.value = projection.nodes;
        context.editSnapshot.value = projection.editSnapshot;
      }
    }
    if (result.assetCleanupCandidates.length) {
      await cleanupDeletedSpaceAssets(result.assetCleanupCandidates);
    }
  }

  function patchActiveNodeData(spaceId: string, id: string, data: SpaceNodePayload) {
    if (context.activeSpaceId.value !== spaceId) return;
    context.nodes.value = context.nodes.value.map((node) =>
      node.id === id ? applySpaceNodeData(node, data) : node,
    );
  }

  return {
    applyNodeDataForSpace,
    discardPendingContent,
    saveNodeData,
    saveNodeDataForSpace,
    savePendingNodeDataForSpace,
  };
}
