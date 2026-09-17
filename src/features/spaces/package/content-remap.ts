import { isString, isUndefined, omitBy } from "es-toolkit";

import type { GroundedCitation } from "../knowledge/model/grounded-citation";
import type { SpaceObjectFieldDefinition } from "../knowledge/model/object-types";
import type { SpaceSavedViewFilter } from "../knowledge/model/projections";
import type { SpaceNodePayload } from "../model/nodes/node-payload";
import { remapTypeId, requiredMap, type PackageIdMaps } from "./id-maps";
import type { SpacePackageManifest } from "./manifest-schema";

type PackageObjectType = SpacePackageManifest["objectTypes"][number];

export function remapObjectTypes(types: readonly PackageObjectType[], maps: PackageIdMaps) {
  return types.map((type) => ({
    id: requiredMap(maps.types, type.id, "object type"),
    name: type.name,
    icon: type.icon,
    description: type.description,
    fields: type.fields.map(remapObjectField),
    createdAt: type.createdAt,
    updatedAt: type.updatedAt,
  }));
}

export function remapQueryFilter(
  filter: SpacePackageManifest["scenes"][number]["queryFilter"],
  maps: PackageIdMaps,
): SpaceSavedViewFilter {
  return omitBy(
    {
      text: filter.text,
      tags: filter.tags,
      statuses: filter.statuses,
      relationKinds: filter.relationKinds,
      relationStatuses: filter.relationStatuses,
      objectTypeIds: filter.objectTypeIds?.map((id) => remapTypeId(id, maps)),
    },
    isUndefined,
  ) as SpaceSavedViewFilter;
}

export function remapNodePayload(
  data: SpaceNodePayload,
  maps: PackageIdMaps,
  objectTypes: readonly PackageObjectType[],
): SpaceNodePayload {
  const copy = structuredClone(data);
  if (copy.sectionId) copy.sectionId = requiredMap(maps.nodes, copy.sectionId, "section");
  if (copy.kind === "object") remapObject(copy, maps, objectTypes);
  if (copy.kind === "portal") remapPortal(copy, maps);
  return copy;
}

function remapPortal(data: Extract<SpaceNodePayload, { kind: "portal" }>, maps: PackageIdMaps) {
  if (data.targetSpaceId !== maps.sourceSpaceId) return;
  data.targetSpaceId = maps.targetSpaceId;
  if (data.targetSceneId) data.targetSceneId = requiredMap(maps.scenes, data.targetSceneId, "portal Scene");
  if (data.targetNodeId) data.targetNodeId = requiredMap(maps.nodes, data.targetNodeId, "portal node");
}

export function remapGroundedCitation(
  citation: GroundedCitation,
  maps: Pick<PackageIdMaps, "nodes" | "chunks">,
  nodeRevisions: Map<string, string>,
): GroundedCitation {
  if (citation.state === "detached") return citation;
  const oldNodeId = citation.nodeId;
  const nodeId = requiredMap(maps.nodes, oldNodeId, "citation node");
  if (citation.chunkId === `node:${oldNodeId}`) {
    const revision = nodeRevisions.get(nodeId);
    if (!revision) throw new Error("Canonical citation node is missing an authoritative Knowledge revision");
    return {
      ...citation,
      nodeId,
      chunkId: `node:${nodeId}`,
      sourceRevision: revision,
      locator: { ...citation.locator, sourceRevision: revision },
    };
  }
  return { ...citation, nodeId, chunkId: requiredMap(maps.chunks, citation.chunkId, "citation chunk") };
}

function remapObject(
  data: Extract<SpaceNodePayload, { kind: "object" }>,
  maps: PackageIdMaps,
  types: readonly PackageObjectType[],
) {
  const sourceTypeId = data.objectTypeId;
  data.objectTypeId = remapTypeId(sourceTypeId, maps);
  const type = types.find((candidate) => candidate.id === sourceTypeId);
  if (!sourceTypeId.startsWith("system:") && !type) {
    throw new Error("Package object references an unknown custom Knowledge Type");
  }
  for (const field of type?.fields ?? []) {
    if (field.kind !== "node-ref") continue;
    const current = data.properties[field.key];
    if (isString(current)) {
      data.properties[field.key] = requiredMap(maps.nodes, current, `node-ref property '${field.key}'`);
    }
  }
}

function remapObjectField(field: PackageObjectType["fields"][number]): SpaceObjectFieldDefinition {
  return omitBy(
    {
      key: field.key,
      label: field.label,
      kind: field.kind,
      required: field.required,
      options: field.options,
    },
    isUndefined,
  ) as SpaceObjectFieldDefinition;
}
