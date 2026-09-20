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
import { permanentlyDeleteNodes, setNodesDeleted, updateNodePayload } from "../../nodes/node-repository";

export function createNodeDataMutations(context: SpaceMutationContext) {
  async function saveNodeData(
    id: string,
    data: SpaceNodePayload,
    sourceChunks?: readonly KnowledgeChunkSeed[],
  ) {
    const releaseMutation = beginSpaceMutation(context.history);
    try {
      return await saveNodeDataForSpace(requireActiveSpaceId(context.activeSpaceId), id, data, sourceChunks);
    } finally {
      releaseMutation();
    }
  }

  async function saveNodeDataForSpace(
    spaceId: string,
    id: string,
    data: SpaceNodePayload,
    sourceChunks?: readonly KnowledgeChunkSeed[],
  ) {
    await savePendingNodeDataForSpace(spaceId, id, data, sourceChunks);
    scheduleKnowledgeIndex(spaceId, [id]);
    void resumePendingEnrichmentJobs({ applyNodeDataForSpace }, spaceId);
  }

  async function savePendingNodeDataForSpace(
    spaceId: string,
    id: string,
    data: SpaceNodePayload,
    sourceChunks?: readonly KnowledgeChunkSeed[],
  ) {
    const validated = prepareSpaceKnowledgePayload(validateSpaceNodePayload(data));
    await updateNodePayload(spaceId, id, validated, sourceChunks);
    patchActiveNodeData(spaceId, id, validated);
  }

  function applyNodeDataForSpace(spaceId: string, id: string, data: SpaceNodePayload) {
    const validated = validateSpaceNodePayload(data);
    patchActiveNodeData(spaceId, id, validated);
    scheduleKnowledgeIndex(spaceId, [id]);
  }

  async function discardPendingItem(spaceId: string, id: string) {
    await setNodesDeleted(spaceId, [id], true);
    const paths = await permanentlyDeleteNodes([{ spaceId, ids: [id] }]);
    if (context.activeSpaceId.value === spaceId) {
      context.nodes.value = context.nodes.value.filter((node) => node.id !== id);
    }
    if (paths.length) await cleanupDeletedSpaceAssets(paths);
  }

  function patchActiveNodeData(spaceId: string, id: string, data: SpaceNodePayload) {
    if (context.activeSpaceId.value !== spaceId) return;
    context.nodes.value = context.nodes.value.map((node) =>
      node.id === id ? applySpaceNodeData(node, data) : node,
    );
  }

  return {
    applyNodeDataForSpace,
    discardPendingItem,
    saveNodeData,
    saveNodeDataForSpace,
    savePendingNodeDataForSpace,
  };
}
