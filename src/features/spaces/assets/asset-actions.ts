import { toast } from "vue-sonner";

import { isSpaceManagedAssetPayload } from "../model/nodes/capabilities";
import type { SpaceNodePayload } from "../model/nodes/node-payload";
import { openSpaceAsset } from "./asset-operations";

export function useSpaceAssetActions(getData: () => SpaceNodePayload) {
  async function openAsset() {
    const data = getData();
    if (!isSpaceManagedAssetPayload(data)) return;
    try {
      await openSpaceAsset(data.assetPath);
    } catch (error) {
      toast.error(String(error));
    }
  }

  return { openAsset };
}
