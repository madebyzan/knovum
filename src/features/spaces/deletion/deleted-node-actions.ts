import { queryClient } from "../../../foundation/query/query-client";
import { cleanupDeletedSpaceAssets } from "../assets/asset-operations";
import { spaceKnowledgeQueryKeys } from "../knowledge/queries/query-options";
import { permanentlyDeleteNodes, restoreNodes } from "../nodes/node-repository";
import type { SpaceDeletedNode } from "./deleted-node";

export async function restoreDeletedNode(spaceId: string, nodeId: string) {
  await restoreNodes(spaceId, [nodeId]);
  await refreshDeletedNodes();
}

export async function permanentlyDeleteItems(items: readonly SpaceDeletedNode[]) {
  const grouped = new Map<string, string[]>();
  for (const item of items) {
    const ids = grouped.get(item.spaceId);
    if (ids) ids.push(item.id);
    else grouped.set(item.spaceId, [item.id]);
  }
  try {
    const assetPaths = await permanentlyDeleteNodes([...grouped].map(([spaceId, ids]) => ({ spaceId, ids })));
    if (assetPaths.length) await cleanupDeletedSpaceAssets(assetPaths);
  } finally {
    await refreshDeletedNodes();
  }
}

export function refreshDeletedNodes() {
  return queryClient.invalidateQueries({ queryKey: [...spaceKnowledgeQueryKeys.root, "deleted"] });
}
