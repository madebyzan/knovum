import { invoke } from "@tauri-apps/api/core";

import { queryClient } from "../../../foundation/query/query-client";
import { cleanupDeletedSpaceAssets } from "../assets/asset-operations";
import { spaceKnowledgeQueryKeys } from "../knowledge/queries/query-options";
import { invokeConfirmedMutation } from "../organization/confirmed-mutation";
import type { SpaceEditReceipt } from "../organization/space-edit-command";
import { spaceEditRepository } from "../structure/space-edit-repository";
import type { SpaceDeletedStructureRoot } from "./deleted-structure";

export function listRecentlyDeletedStructure(spaceId: string | null) {
  return invoke<SpaceDeletedStructureRoot[]>("plugin:spaces|list_recently_deleted_structure", {
    spaceId,
  });
}

export async function restoreDeletedStructure(deletedRoot: SpaceDeletedStructureRoot, wholeBatch: boolean) {
  const mutationId = crypto.randomUUID();
  const snapshot = await spaceEditRepository.load(deletedRoot.spaceId);
  await invokeConfirmedMutation<SpaceEditReceipt>(
    "plugin:spaces|restore_deleted_structure",
    {
      request: {
        spaceId: deletedRoot.spaceId,
        mutationId,
        expectedEditRevision: snapshot.editRevision,
        batchId: deletedRoot.batchId,
        rootStructureNodeId: wholeBatch ? null : deletedRoot.rootStructureNodeId,
      },
    },
    mutationId,
  );
  await refreshDeletedStructure();
}

export async function permanentlyDeleteStructure(
  deletedRoots: readonly SpaceDeletedStructureRoot[],
  wholeBatches: boolean,
) {
  const batches = new Map<string, SpaceDeletedStructureRoot[]>();
  for (const deletedRoot of deletedRoots) {
    const key = `${deletedRoot.spaceId}\u0000${deletedRoot.batchId}`;
    batches.set(key, [...(batches.get(key) ?? []), deletedRoot]);
  }
  try {
    const paths = (
      await Promise.all([...batches.values()].map((roots) => permanentlyDeleteBatch(roots, wholeBatches)))
    ).flat();
    if (paths.length) await cleanupDeletedSpaceAssets([...new Set(paths)]);
  } finally {
    await refreshDeletedStructure();
  }
}

function permanentlyDeleteBatch(roots: readonly SpaceDeletedStructureRoot[], wholeBatch: boolean) {
  const first = roots[0]!;
  return invoke<string[]>("plugin:spaces|permanently_delete_structure", {
    request: {
      spaceId: first.spaceId,
      batchId: first.batchId,
      rootStructureNodeId: wholeBatch ? null : first.rootStructureNodeId,
    },
  });
}

export function refreshDeletedStructure() {
  return queryClient.invalidateQueries({ queryKey: [...spaceKnowledgeQueryKeys.root, "deleted"] });
}
