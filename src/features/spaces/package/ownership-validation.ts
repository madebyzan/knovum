import type { SpacePackageManifest } from "./manifest-schema";

export function validateManifestOwnership(manifest: SpacePackageManifest) {
  validateSpaceOwnedContent(manifest);
  validateObjectTypeOwnership(manifest);
}

function validateSpaceOwnedContent(manifest: SpacePackageManifest) {
  const groups: Array<readonly { spaceId?: string | null }[]> = [
    manifest.scenes,
    manifest.relations,
    manifest.derivations,
    manifest.chunks,
    manifest.provenance,
    manifest.knowledgeEvents,
  ];
  for (const group of groups) validateOwnedGroup(group, manifest.space.id);
}

function validateOwnedGroup(rows: readonly { spaceId?: string | null }[], spaceId: string) {
  for (const row of rows) {
    if (row.spaceId !== spaceId) throw new Error("Space package contains a row owned by another Space");
  }
}

function validateObjectTypeOwnership(manifest: SpacePackageManifest) {
  for (const type of manifest.objectTypes) {
    if (type.spaceId !== null && type.spaceId !== manifest.space.id) {
      throw new Error("Space package contains an Object Type owned by another Space");
    }
  }
}
