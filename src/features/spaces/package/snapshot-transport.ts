import { invokeCommand as invoke } from "../../../foundation/ipc/invoke-command";
import type { SpacePackageAsset } from "./contract";
import type { SpacePackageManifest } from "./manifest-schema";

interface SpacePackageSnapshot {
  manifest: SpacePackageManifest;
  assets: SpacePackageAsset[];
}

export async function collectSpacePackageSnapshot(spaceId: string): Promise<SpacePackageSnapshot> {
  return invoke<SpacePackageSnapshot>("plugin:spaces|collect_space_package_snapshot", {
    spaceId,
  });
}
