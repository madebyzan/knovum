import { groupBy, uniq } from "es-toolkit";
import pLimit from "p-limit";

import { invokeCommand } from "../../../foundation/ipc/invoke-command";
import { queryClient } from "../../../foundation/query/query-client";
import { cleanupDeletedSpaceAssets } from "../assets/asset-operations";
import type { SpaceEditReceipt } from "../contracts/generated/SpaceEditReceipt";
import { spaceKnowledgeQueryKeys } from "../knowledge/queries/query-options";
import { invokeConfirmedMutation } from "../structure/confirmed-mutation";
import { spaceEditOperations } from "../structure/space-edit-operations";
import type { SpaceDeletedStructureRoot } from "./deleted-structure";

const permanentDeletions = pLimit(2);

export function listRecentlyDeletedStructure(spaceId: string | null) {
  return invokeCommand<SpaceDeletedStructureRoot[]>("plugin:spaces|list_recently_deleted_structure", {
    spaceId,
  });
}

export async function restoreDeletedStructure(deletedRoot: SpaceDeletedStructureRoot, wholeBatch: boolean) {
  const mutationId = crypto.randomUUID();
  const snapshot = await spaceEditOperations.load(deletedRoot.spaceId);
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
  const batches = groupBy(deletedRoots, ({ spaceId, batchId }) => `${spaceId}\u0000${batchId}`);
  try {
    const paths = (
      await Promise.all(
        Object.values(batches).map((roots) =>
          permanentDeletions(() => permanentlyDeleteBatch(roots, wholeBatches)),
        ),
      )
    ).flat();
    if (paths.length) await cleanupDeletedSpaceAssets(uniq(paths));
  } finally {
    await refreshDeletedStructure();
  }
}

function permanentlyDeleteBatch(roots: readonly SpaceDeletedStructureRoot[], wholeBatch: boolean) {
  const first = roots[0]!;
  return invokeCommand<string[]>("plugin:spaces|delete_structure_permanently", {
    request: {
      spaceId: first.spaceId,
      batchId: first.batchId,
      rootStructureNodeId: wholeBatch ? null : first.rootStructureNodeId,
    },
  });
}

function refreshDeletedStructure() {
  return queryClient.invalidateQueries({ queryKey: [...spaceKnowledgeQueryKeys.root, "deleted"] });
}
