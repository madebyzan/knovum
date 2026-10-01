import { getErrorMessage } from "../../../../foundation/errors/error-format";
import { useTaskStore } from "../../../../foundation/tasks/task-store";
import { cleanupDeletedSpaceAssets } from "../../assets/asset-operations";
import { applySpaceNodeContent } from "../../canvas/node";
import { spaceNodeSizePolicy, spaceNodeSupportsContentSizing } from "../../canvas/node-size";
import { refreshEnrichmentTasks } from "../../enrichment/job-observation";
import { beginSpaceMutation } from "../../history/mutation-barrier";
import { requireActiveSpaceId, type SpaceMutationContext } from "../../history/mutation-context";
import { scheduleKnowledgeIndex } from "../../knowledge/indexing/index-scheduler";
import type { KnowledgeChunkSeed } from "../../knowledge/model/schema/chunk";
import type { SpaceNodePayload } from "../../model/nodes/node-payload";
import { validateSpaceNodePayload } from "../../model/nodes/node-validation";
import {
  discardPendingContent as persistPendingContentDiscard,
  updateNodePayload,
  type NodePayloadChangeInput,
} from "../../nodes/node-operations";
import { spaceEditOperations } from "../../structure/space-edit-operations";
import { loadSpaceSessionProjection } from "../space-projection";

export function createNodeContentMutations(context: SpaceMutationContext) {
  async function refreshCommittedGeometry(spaceId: string) {
    const snapshot = await spaceEditOperations.load(spaceId);
    if (
      context.activeSpaceId.value === spaceId &&
      context.editSnapshot.value.editRevision < snapshot.editRevision
    )
      context.editSnapshot.value = snapshot;
  }

  async function saveNodeContent(
    id: string,
    expectedContent: SpaceNodePayload,
    data: SpaceNodePayload,
    sourceChunks?: readonly KnowledgeChunkSeed[],
  ) {
    const releaseMutation = beginSpaceMutation(context.history);
    try {
      const spaceId = requireActiveSpaceId(context.activeSpaceId);
      await persistNodeContent(spaceId, { id, expectedData: expectedContent, data, sourceChunks });
      afterSaved(spaceId, id);
    } finally {
      releaseMutation();
    }
  }

  async function saveNodeContentForSpace(
    spaceId: string,
    id: string,
    expectedContent: SpaceNodePayload,
    data: SpaceNodePayload,
  ) {
    await persistNodeContent(spaceId, { id, expectedData: expectedContent, data });
    afterSaved(spaceId, id);
  }

  async function savePendingNodeContentForSpace(
    spaceId: string,
    id: string,
    expectedContent: SpaceNodePayload,
    data: SpaceNodePayload,
  ) {
    await persistNodeContent(spaceId, { id, expectedData: expectedContent, data });
  }

  async function persistNodeContent(spaceId: string, change: NodePayloadChangeInput) {
    const validated = validateSpaceNodePayload(change.data);
    const committed = await updateNodePayload(spaceId, { ...change, data: validated });
    patchActiveNodeContent(spaceId, change.id, committed.data);
    if (
      committed.editSnapshot &&
      context.activeSpaceId.value === spaceId &&
      committed.editSnapshot.editRevision > context.editSnapshot.value.editRevision
    )
      context.editSnapshot.value = committed.editSnapshot;
  }

  function applyNodeContentForSpace(spaceId: string, id: string, data: SpaceNodePayload) {
    const validated = validateSpaceNodePayload(data);
    const previous = context.nodes.value.find((node) => node.id === id)?.data;
    patchActiveNodeContent(spaceId, id, validated);
    if (
      context.activeSpaceId.value === spaceId &&
      previous &&
      spaceNodeSupportsContentSizing(validated) &&
      spaceNodeSizePolicy(previous).contentAspect?.ratio !==
        spaceNodeSizePolicy(validated).contentAspect?.ratio
    )
      void refreshCommittedGeometry(spaceId).catch((error: unknown) => {
        const tasks = useTaskStore();
        tasks.fail(tasks.start(validated.title), getErrorMessage(error));
      });
    scheduleKnowledgeIndex(spaceId, [id]);
  }

  async function discardPendingContent(spaceId: string, id: string) {
    const snapshot =
      context.activeSpaceId.value === spaceId
        ? context.editSnapshot.value
        : await spaceEditOperations.load(spaceId);
    const discardReceipt = await persistPendingContentDiscard({
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
    if (discardReceipt.assetCleanupCandidates.length) {
      await cleanupDeletedSpaceAssets(discardReceipt.assetCleanupCandidates);
    }
  }

  function patchActiveNodeContent(spaceId: string, id: string, data: SpaceNodePayload) {
    if (context.activeSpaceId.value !== spaceId) return;
    context.nodes.value = context.nodes.value.map((node) =>
      node.id === id ? applySpaceNodeContent(node, data) : node,
    );
  }

  return {
    applyNodeContentForSpace,
    discardPendingContent,
    saveNodeContent,
    saveNodeContentForSpace,
    savePendingNodeContentForSpace,
  };
}

function afterSaved(spaceId: string, id: string) {
  scheduleKnowledgeIndex(spaceId, [id]);
  void refreshEnrichmentTasks(spaceId);
}
