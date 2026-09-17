import { warn as logWarn } from "@tauri-apps/plugin-log";

import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import { isSpaceManagedAssetPayload } from "../model/nodes/capabilities";
import type { SpaceNodePayload } from "../model/nodes/node-payload";
import { refreshSpaceAsset } from "./asset-operations";

export async function refreshSpaceResources(
  spaceId: string,
  nodes: readonly SpaceCanvasNode[],
  apply: (spaceId: string, id: string, data: SpaceNodePayload) => void,
) {
  let failed = 0;
  for (const node of nodes) {
    if (!isSpaceManagedAssetPayload(node.data)) continue;
    if (node.data.processingState === "importing" || node.data.processingState === "processing") {
      continue;
    }
    try {
      // oxlint-disable-next-line eslint/no-await-in-loop -- Refreshes apply in node order and must not burst filesystem work.
      const refreshed = await refreshSpaceAsset(spaceId, node.id);
      if (refreshed) apply(spaceId, node.id, refreshed);
    } catch (error) {
      void logWarn(`Space resource refresh failed for ${node.id}: ${String(error)}`);
      failed += 1;
    }
  }
  return failed;
}
