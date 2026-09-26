import { validatePackageDerivations } from "./derivation-validation";
import { validatePackageKnowledgeSemantics } from "./knowledge-validation";
import { manifestSchema, type SpacePackageManifest } from "./manifest-schema";
import { validateManifestOwnership } from "./ownership-validation";
import { validatePackageRelationSemantics } from "./relation-validation";
import { validatePackageStructure } from "./structure-validation";

export function parseSpacePackageManifest(value: string): SpacePackageManifest {
  const manifest = manifestSchema.parse(JSON.parse(value));
  validatePackageStructure(manifest);
  validateManifestOwnership(manifest);
  validatePackageRelationSemantics(manifest);
  validatePackageDerivations(manifest);
  validatePackageKnowledgeSemantics(manifest);
  return manifest;
}
