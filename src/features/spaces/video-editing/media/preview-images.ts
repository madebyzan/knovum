import { versionedMediaSource } from "../../media/media-element-source";
import type { VideoMaterialNode } from "../material";
import type { VideoAsset } from "../project";
import { createAssetCache } from "./asset-cache";

/**
 * Keeps one decoded bitmap per image asset. Bitmaps are drawn by the stage scene like a video
 * frame, so still images take part in the same transforms, fades and transitions.
 */
export function createImageLibrary(materials: () => ReadonlyMap<string, VideoMaterialNode>) {
  const controller = new AbortController();

  async function open(asset: VideoAsset) {
    const node = materials().get(asset.nodeId);
    const url = versionedMediaSource(node?.data.assetPath, asset.sourceRevision);
    if (!url) throw new Error("Image material is unavailable");
    const response = await fetch(url, { signal: controller.signal });
    if (!response.ok) throw new Error("Cannot read the selected image");
    return createImageBitmap(await response.blob());
  }

  const cache = createAssetCache(open);
  return {
    ready: cache.ready,
    load: cache.load,
    forget: (asset: VideoAsset) => cache.forget(asset)?.close(),
    dispose() {
      controller.abort();
      for (const bitmap of cache.opened()) bitmap.close();
      cache.clear();
    },
  };
}

export type ImageLibrary = ReturnType<typeof createImageLibrary>;
