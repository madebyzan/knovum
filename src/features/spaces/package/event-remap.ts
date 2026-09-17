import { isPlainObject, isString } from "es-toolkit";

import { remapTypeId, requiredMap, type PackageIdMaps } from "./id-maps";

export function remapEventEntity(kind: string, id: string, maps: PackageIdMaps) {
  switch (kind) {
    case "node":
      return requiredMap(maps.nodes, id, "event node");
    case "relation":
      return requiredMap(maps.relations, id, "event relation");
    case "provenance":
      return requiredMap(maps.provenance, id, "event provenance");
    case "type":
      return remapTypeId(id, maps);
    default:
      throw new Error(`Unsupported package event entity kind '${kind}'`);
  }
}

export function remapEventPayload(
  entityKind: string,
  eventKind: string,
  payload: unknown,
  maps: PackageIdMaps,
): unknown {
  if (!isPlainObject(payload)) return payload;
  const copy = structuredClone(payload) as Record<string, unknown>;
  if (entityKind === "provenance" && eventKind === "created") remapProvenancePayload(copy, maps);
  return copy;
}

function remapProvenancePayload(payload: Record<string, unknown>, maps: PackageIdMaps) {
  const targetKind = payload.targetKind;
  if (targetKind !== "node" && targetKind !== "relation") {
    throw new Error("Package provenance event has an invalid targetKind");
  }
  if (!isString(payload.targetId)) throw new Error("Package provenance event targetId is required");
  payload.targetId =
    targetKind === "node"
      ? requiredMap(maps.nodes, payload.targetId, "provenance event target")
      : requiredMap(maps.relations, payload.targetId, "provenance event target");
}
