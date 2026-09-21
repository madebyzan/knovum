import { isString } from "es-toolkit";

import type { SpaceNodePayload } from "../model/nodes/node-payload";
import type { Space } from "../model/space";
import { remapGroundedCitation, remapObjectTypes, remapQueryFilter } from "./content-remap";
import { remapEventEntity, remapEventPayload } from "./event-remap";
import { referenceMap, requiredMap } from "./id-maps";
import { restorePackageNodes } from "./node-restore";
import type { SpacePackageRestoreContext } from "./restore-context";
import { packageSpace, restoreOwnedRow } from "./restore-row";
import type { SpacePackageRestoreRequest } from "./restore-transport";
import { restorePackageStructure } from "./structure-restore";
import { restoreWorkflowRuns } from "./workflow-run-restore";

export function buildSpacePackageRestoreRequest(
  context: SpacePackageRestoreContext,
  space: Space,
): SpacePackageRestoreRequest {
  const nodes = restorePackageNodes(context);
  const nodeRevisions = new Map(
    nodes.flatMap((node) => {
      const data = node.data as SpaceNodePayload;
      return isString(data.knowledgeRevision) ? [[node.id, data.knowledgeRevision] as const] : [];
    }),
  );
  return {
    space: packageSpace(space),
    ...restorePackageStructure(context),
    objectTypes: remapObjectTypes(context.manifest.objectTypes, context.maps),
    nodes,
    scenes: restoreScenes(context),
    relations: restoreRelations(context),
    derivations: restoreDerivations(context),
    creationWorkflowRuns: restoreWorkflowRuns(context),
    provenance: restoreProvenance(context),
    knowledgeEvents: restoreKnowledgeEvents(context),
    chunks: restoreChunks(context),
    threads: restoreThreads(context),
    messages: restoreMessages(context, nodeRevisions),
  };
}

function restoreDerivations(context: SpacePackageRestoreContext): SpacePackageRestoreRequest["derivations"] {
  const { maps } = context;
  return context.manifest.derivations.map((derivation) => {
    const { groundedDocument, generatedImage, generatedVideo, ...row } = restoreOwnedRow(
      derivation,
      maps.sourceSpaceId,
    );
    return {
      ...row,
      id: requiredMap(maps.derivations, derivation.id, "knowledge derivation"),
      outputNodeId: requiredMap(maps.nodes, derivation.outputNodeId, "derivation output"),
      inputs: derivation.inputs.map((input) => ({
        ...input,
        nodeId: input.nodeId === null ? null : requiredMap(maps.nodes, input.nodeId, "derivation input"),
      })),
      ...(groundedDocument ? { groundedDocument } : {}),
      ...(generatedImage ? { generatedImage } : {}),
      ...(generatedVideo ? { generatedVideo } : {}),
    };
  });
}

function restoreScenes(context: SpacePackageRestoreContext): SpacePackageRestoreRequest["scenes"] {
  const { maps } = context;
  return context.manifest.scenes.map((scene) => ({
    id: requiredMap(maps.scenes, scene.id, "Scene"),
    title: scene.title,
    thumbnailPath: null,
    viewportX: scene.viewportX,
    viewportY: scene.viewportY,
    viewportZoom: scene.viewportZoom,
    orderIndex: scene.orderIndex,
    focusStructureNodeIds: scene.focusStructureNodeIds.map((id) => referenceMap(maps.structureNodes, id)),
    relationKinds: scene.relationKinds,
    relationStatuses: scene.relationStatuses,
    queryFilter: remapQueryFilter(scene.queryFilter, maps),
    createdAt: scene.createdAt,
    updatedAt: scene.updatedAt,
  }));
}

function restoreRelations(context: SpacePackageRestoreContext): SpacePackageRestoreRequest["relations"] {
  const { maps } = context;
  return context.manifest.relations.map((relation) => ({
    ...restoreOwnedRow(relation, maps.sourceSpaceId),
    id: requiredMap(maps.relations, relation.id, "relation"),
    sourceId: requiredMap(maps.nodes, relation.sourceId, "relation source"),
    targetId: requiredMap(maps.nodes, relation.targetId, "relation target"),
    confidence: relation.confidence ?? null,
    actorId: relation.actorId ?? null,
  }));
}

function restoreChunks(context: SpacePackageRestoreContext): SpacePackageRestoreRequest["chunks"] {
  return context.manifest.chunks.map((source) => {
    const chunk = restoreOwnedRow(source, context.maps.sourceSpaceId);
    return {
      ...chunk,
      id: requiredMap(context.maps.chunks, chunk.id, "chunk"),
      nodeId: requiredMap(context.maps.nodes, chunk.nodeId, "chunk node"),
      metadata: chunk.metadata ?? {},
    };
  });
}

function restoreThreads(context: SpacePackageRestoreContext): SpacePackageRestoreRequest["threads"] {
  return context.manifest.aiThreads.map((source) => {
    const thread = restoreOwnedRow(source, context.maps.sourceSpaceId);
    return {
      ...thread,
      id: requiredMap(context.maps.threads, thread.id, "AI thread"),
      scopeNodeIds: thread.scopeNodeIds.map((id) => requiredMap(context.maps.nodes, id, "AI scope node")),
    };
  });
}

function restoreMessages(
  context: SpacePackageRestoreContext,
  nodeRevisions: Map<string, string>,
): SpacePackageRestoreRequest["messages"] {
  return context.manifest.aiMessages.map((message) => {
    const { provider, model, ...content } = message;
    return {
      ...content,
      id: crypto.randomUUID(),
      threadId: requiredMap(context.maps.threads, message.threadId, "AI message thread"),
      citations: message.citations.map((citation) =>
        remapGroundedCitation(citation, context.maps, nodeRevisions),
      ),
      ...(provider === undefined ? {} : { provider }),
      ...(model === undefined ? {} : { model }),
    };
  });
}

function restoreProvenance(context: SpacePackageRestoreContext): SpacePackageRestoreRequest["provenance"] {
  const { maps } = context;
  return context.manifest.provenance.map((source) => {
    const { actorId, ...provenance } = restoreOwnedRow(source, maps.sourceSpaceId);
    return {
      ...provenance,
      id: requiredMap(maps.provenance, provenance.id, "provenance"),
      targetId:
        provenance.targetKind === "node"
          ? requiredMap(maps.nodes, provenance.targetId, "provenance node")
          : requiredMap(maps.relations, provenance.targetId, "provenance relation"),
      sourceNodeId: requiredMap(maps.nodes, provenance.sourceNodeId, "provenance source"),
      ...(actorId == null ? {} : { actorId }),
    };
  });
}

function restoreKnowledgeEvents(
  context: SpacePackageRestoreContext,
): SpacePackageRestoreRequest["knowledgeEvents"] {
  return context.manifest.knowledgeEvents.map((source) => {
    const { actorId, ...event } = restoreOwnedRow(source, context.maps.sourceSpaceId);
    return {
      ...event,
      id: crypto.randomUUID(),
      entityId: remapEventEntity(event.entityKind, event.entityId, context.maps),
      payload: remapEventPayload(event.entityKind, event.eventKind, event.payload, context.maps),
      ...(actorId == null ? {} : { actorId }),
    };
  });
}
