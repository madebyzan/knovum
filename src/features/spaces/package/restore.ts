import type { Space } from "../model/space";
import type { ImportedSpacePackageAsset } from "./contract";
import { createPackageIdMaps } from "./id-maps";
import { parseSpacePackageManifest } from "./manifest-parser";
import type { SpacePackageManifest } from "./manifest-schema";
import { buildSpacePackageRestoreRequest } from "./restore-request";
import { restoreSpacePackageData } from "./restore-transport";

export async function restoreSpacePackage(manifestJson: string, importedAssets: ImportedSpacePackageAsset[]) {
  const manifest = parseSpacePackageManifest(manifestJson);
  const nodeCount = manifest.nodes.length;
  const space = importedSpace(manifest.space, nodeCount);
  const maps = createPackageIdMaps(manifest, space.id);
  const context = {
    manifest,
    maps,
    assets: new Map(importedAssets.map((asset) => [asset.token, asset.path])),
  };
  await restoreSpacePackageData(buildSpacePackageRestoreRequest(context, space));
  return space;
}

function importedSpace(source: SpacePackageManifest["space"], nodeCount: number): Space {
  const now = Date.now();
  return {
    ...source,
    id: crypto.randomUUID(),
    kind: "space",
    archivedAt: null,
    favorite: false,
    thumbnailPath: null,
    nodeCount,
    lastOpenedAt: now,
    updatedAt: now,
  };
}
