import type { Space } from "../model/space";
import type { CanvasPackageCodec } from "./canvas-state-codec";
import type { ImportedSpacePackageAsset } from "./contract";
import { createPackageIdMaps } from "./id-maps";
import { parseSpacePackageManifest } from "./manifest-parser";
import type { SpacePackageManifest } from "./manifest-schema";
import { buildSpacePackageRestoreRequest } from "./restore-request";
import { restoreSpacePackageData } from "./restore-transport";

export async function restoreSpacePackage(
  manifestJson: string,
  importedAssets: ImportedSpacePackageAsset[],
  canvasCodec: CanvasPackageCodec,
) {
  const manifest = parseSpacePackageManifest(manifestJson, canvasCodec);
  const nodeCount = manifest.nodes.filter((node) => node.kind !== "section").length;
  const space = importedSpace(manifest.space, nodeCount);
  const maps = createPackageIdMaps(manifest, space.id);
  const context = {
    manifest,
    maps,
    assets: new Map(importedAssets.map((item) => [item.token, item.path])),
  };
  await restoreSpacePackageData(buildSpacePackageRestoreRequest(context, space, canvasCodec));
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
