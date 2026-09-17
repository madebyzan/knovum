import { compareRelationIdsBinary, isSymmetricRelationKind } from "../model/relations/relation";
import type { SpacePackageManifest } from "./manifest-schema";

export function validatePackageRelationSemantics(manifest: SpacePackageManifest) {
  const current = new Set<string>();
  for (const relation of manifest.relations) validateRelation(relation, current);
}

function validateRelation(relation: SpacePackageManifest["relations"][number], current: Set<string>) {
  if (!isSymmetricRelationKind(relation.kind)) return;
  validateCanonicalEndpoints(relation.sourceId, relation.targetId);
  if (relation.status === "invalidated") return;
  const key = `${relation.kind}\u0000${relation.sourceId}\u0000${relation.targetId}`;
  if (current.has(key)) throw new Error("Space package contains a duplicate current symmetric Relation");
  current.add(key);
}

function validateCanonicalEndpoints(sourceId: string, targetId: string) {
  if (compareRelationIdsBinary(sourceId, targetId) > 0) {
    throw new Error("Space package symmetric Relations must use canonical endpoints");
  }
}
