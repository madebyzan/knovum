import { remapObjectTypes, remapQueryFilter } from "./content-remap";
import { remapEventEntity, remapEventPayload } from "./event-remap";
import { referenceMap, requiredMap } from "./id-maps";
import type { SpacePackageRestoreContext } from "./restore-context";
import { restoreOwnedRow } from "./restore-row";
import type { SpacePackageRestoreRequest } from "./restore-transport";

export { remapObjectTypes };

export function restoreDerivations(
  context: SpacePackageRestoreContext,
): SpacePackageRestoreRequest["derivations"] {
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

export function restoreScenes(context: SpacePackageRestoreContext): SpacePackageRestoreRequest["scenes"] {
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

export function restoreRelations(
  context: SpacePackageRestoreContext,
): SpacePackageRestoreRequest["relations"] {
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

export function restoreChunks(context: SpacePackageRestoreContext): SpacePackageRestoreRequest["chunks"] {
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

export function restoreProvenance(
  context: SpacePackageRestoreContext,
): SpacePackageRestoreRequest["provenance"] {
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

export function restoreKnowledgeEvents(
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
