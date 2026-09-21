import { isPlainObject } from "es-toolkit";

import { SPACE_PACKAGE_FORMAT, SPACE_PACKAGE_VERSION } from "./contract";
import { validatePackageDerivations } from "./derivation-validation";
import { validatePackageKnowledgeSemantics } from "./knowledge-validation";
import { manifestSchema, type SpacePackageManifest } from "./manifest-schema";
import { validateManifestOwnership } from "./ownership-validation";
import { validatePackageRelationSemantics } from "./relation-validation";
import { validatePackageStructure } from "./structure-validation";
import { validatePackageWorkflowRuns } from "./workflow-run-validation";

export function parseSpacePackageManifest(value: string): SpacePackageManifest {
  const parsed: unknown = JSON.parse(value);
  if (!isPlainObject(parsed)) {
    throw new Error("Space package manifest must be an object");
  }
  const source = parsed as Record<string, unknown>;
  if (source.format !== SPACE_PACKAGE_FORMAT) throw new Error("Unsupported Space package format");
  if (source.version !== SPACE_PACKAGE_VERSION) {
    throw new Error(`Unsupported Space package version: ${String(source.version)}`);
  }
  const manifest = manifestSchema.parse(source);
  validatePackageStructure(manifest);
  validateManifestOwnership(manifest);
  validatePackageRelationSemantics(manifest);
  validatePackageDerivations(manifest);
  validatePackageWorkflowRuns(manifest);
  validatePackageKnowledgeSemantics(manifest);
  return manifest;
}
