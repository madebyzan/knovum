import type { SpacePackageManifest } from "./manifest-schema";
import { validatePackagedStructure } from "./structure-validation";

export function validateManifestOwnership(manifest: SpacePackageManifest) {
  validatePackagedStructure(manifest);
  validateSpaceOwnedRows(manifest);
  validateObjectTypeOwnership(manifest);
  const nodeIds = new Set(manifest.nodes.map((node) => node.id));
  validateThreadScopes(manifest, nodeIds);
  validateMessageThreads(manifest);
}

function validateSpaceOwnedRows(manifest: SpacePackageManifest) {
  const groups: Array<readonly { spaceId?: string | null }[]> = [
    manifest.scenes,
    manifest.relations,
    manifest.derivations,
    manifest.chunks,
    manifest.aiThreads,
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

function validateThreadScopes(manifest: SpacePackageManifest, nodeIds: Set<string>) {
  for (const thread of manifest.aiThreads) {
    if (thread.scopeNodeIds.some((id) => !nodeIds.has(id))) {
      throw new Error("Space package Grounded AI scope references an unavailable node");
    }
  }
}

function validateMessageThreads(manifest: SpacePackageManifest) {
  const threadIds = new Set(manifest.aiThreads.map((thread) => thread.id));
  for (const message of manifest.aiMessages) {
    if (!threadIds.has(message.threadId))
      throw new Error("Space package contains an AI message for an unknown thread");
  }
}
