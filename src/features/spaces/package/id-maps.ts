import type { SpacePackageManifest } from "./manifest-schema";

export interface PackageIdMaps {
  sourceSpaceId: string;
  targetSpaceId: string;
  nodes: Map<string, string>;
  structureNodes: Map<string, string>;
  scenes: Map<string, string>;
  relations: Map<string, string>;
  derivations: Map<string, string>;
  runs: Map<string, string>;
  runSteps: Map<string, string>;
  runAttempts: Map<string, string>;
  externalAgentConversations: Map<string, string>;
  externalAgentTurns: Map<string, string>;
  assistantConversations: Map<string, string>;
  assistantMessages: Map<string, string>;
  types: Map<string, string>;
  provenance: Map<string, string>;
  chunks: Map<string, string>;
}

export function createPackageIdMaps(manifest: SpacePackageManifest, targetSpaceId: string): PackageIdMaps {
  return {
    sourceSpaceId: manifest.space.id,
    targetSpaceId,
    nodes: idMap(manifest.nodes),
    structureNodes: idMap(manifest.structureNodes),
    scenes: idMap(manifest.scenes),
    relations: idMap(manifest.relations),
    derivations: idMap(manifest.derivations),
    runs: idMap(manifest.runs),
    runSteps: idMap(manifest.runSteps),
    runAttempts: idMap(manifest.runAttempts),
    externalAgentConversations: idMap(manifest.externalAgentConversations),
    externalAgentTurns: idMap(manifest.externalAgentTurns),
    assistantConversations: idMap(manifest.assistantConversations),
    assistantMessages: idMap(manifest.assistantMessages),
    types: idMap(manifest.objectTypes),
    provenance: idMap(manifest.provenance),
    chunks: idMap(manifest.chunks),
  };
}

export function requiredMap(map: Map<string, string>, key: string, kind: string): string {
  const value = map.get(key);
  if (!value) throw new Error(`Package references an unknown ${kind}`);
  return value;
}

/** Preserve dangling navigation identity so invalid Saved View targets stay explicit after restore. */
export function referenceMap(map: Map<string, string>, key: string): string {
  const existing = map.get(key);
  if (existing) return existing;
  const created = crypto.randomUUID();
  map.set(key, created);
  return created;
}

export function remapTypeId(id: string, maps: PackageIdMaps) {
  return id.startsWith("system:") ? id : requiredMap(maps.types, id, "Knowledge Type");
}

function idMap(rows: readonly { id: string }[]) {
  return new Map(rows.map((row) => [row.id, crypto.randomUUID()]));
}
