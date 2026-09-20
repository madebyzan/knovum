import type { SpacePackageManifest } from "./manifest-schema";

export interface PackageIdMaps {
  sourceSpaceId: string;
  targetSpaceId: string;
  nodes: Map<string, string>;
  items: Map<string, string>;
  scenes: Map<string, string>;
  relations: Map<string, string>;
  derivations: Map<string, string>;
  workflowRuns: Map<string, string>;
  types: Map<string, string>;
  provenance: Map<string, string>;
  chunks: Map<string, string>;
  threads: Map<string, string>;
}

export function createPackageIdMaps(manifest: SpacePackageManifest, targetSpaceId: string): PackageIdMaps {
  const nodes = idMap(manifest.nodes);
  const items = new Map(
    manifest.canvasState.document.structure.items.map((item) => [
      item.id,
      item.kind === "content" ? requiredMap(nodes, item.nodeId, "content") : crypto.randomUUID(),
    ]),
  );
  return {
    sourceSpaceId: manifest.space.id,
    targetSpaceId,
    nodes,
    items,
    scenes: idMap(manifest.scenes),
    relations: idMap(manifest.relations),
    derivations: idMap(manifest.derivations),
    workflowRuns: idMap(manifest.creationWorkflowRuns),
    types: idMap(manifest.objectTypes),
    provenance: idMap(manifest.provenance),
    chunks: idMap(manifest.chunks),
    threads: idMap(manifest.aiThreads),
  };
}

export function requiredMap(map: ReadonlyMap<string, string>, key: string, kind: string): string {
  const value = map.get(key);
  if (!value) throw new Error(`Package references an unknown ${kind}`);
  return value;
}

export function remapTypeId(id: string, maps: PackageIdMaps) {
  return id.startsWith("system:") ? id : requiredMap(maps.types, id, "Knowledge Type");
}

function idMap(rows: readonly { id: string }[]) {
  return new Map(rows.map((row) => [row.id, crypto.randomUUID()]));
}
