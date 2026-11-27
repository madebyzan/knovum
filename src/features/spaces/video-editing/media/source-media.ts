import { ALL_FORMATS, Input, UrlSource, type InputVideoTrack } from "mediabunny";

import { versionedMediaSource } from "../../media/media-element-source";
import type { VideoMaterialNode } from "../material";
import type { VideoAsset } from "../project";
import { assetIdentity, createAssetCache } from "./asset-cache";

interface SourceMedia {
  /** The source's own video track, present only when this WebView can decode it. */
  video: InputVideoTrack | undefined;
  audible: boolean;
}

/**
 * Opens the original file of every asset once per revision. Mediabunny reads only the container
 * index through ranged requests, so what a source holds is known as soon as it is added, whatever
 * its length — no transcode, no full read.
 */
export function createSourceMediaLibrary(materials: () => ReadonlyMap<string, VideoMaterialNode>) {
  const inputsByKey = new Map<string, Input>();

  function disposeKey(key: string) {
    const existing = inputsByKey.get(key);
    if (!existing) return;
    inputsByKey.delete(key);
    existing.dispose();
  }

  async function open(asset: VideoAsset): Promise<SourceMedia> {
    const url = versionedMediaSource(materials().get(asset.nodeId)?.data.assetPath, asset.sourceRevision);
    if (!url) throw new Error("Media material is unavailable");
    const key = assetIdentity(asset);
    disposeKey(key);
    const input = new Input({ formats: ALL_FORMATS, source: new UrlSource(url) });
    inputsByKey.set(key, input);
    const [video, audio] = await Promise.all([input.getPrimaryVideoTrack(), input.getPrimaryAudioTrack()]);
    return { video: video && (await video.canDecode()) ? video : undefined, audible: !!audio };
  }

  const cache = createAssetCache(open);
  return {
    load: cache.load,
    /** Whether the asset carries sound; a still image never does. */
    async audible(asset: VideoAsset) {
      if (asset.kind === "image") return false;
      return asset.kind === "audio" || (await cache.load(asset)).audible;
    },
    forget(asset: VideoAsset) {
      disposeKey(assetIdentity(asset));
      return cache.forget(asset);
    },
    dispose() {
      cache.clear();
      for (const input of inputsByKey.values()) input.dispose();
      inputsByKey.clear();
    },
  };
}

export type SourceMediaLibrary = ReturnType<typeof createSourceMediaLibrary>;
