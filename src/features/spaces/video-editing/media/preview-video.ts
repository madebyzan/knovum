import { ALL_FORMATS, Input, UrlSource, type InputVideoTrack } from "mediabunny";
import { shallowRef } from "vue";

import { versionedMediaSource } from "../../media/media-element-source";
import { prepareVideoEditProxy } from "../prepare-video-edit-proxy";
import type { VideoAsset } from "../project";
import { assetIdentity, createAssetCache } from "./asset-cache";
import { mediaSourceOf } from "./source-ipc";
import type { SourceMediaLibrary } from "./source-media";

export interface PreviewVideo {
  track: InputVideoTrack;
  /** Playback URL for native hardware `<video>` decoding (original source or H.264 proxy). */
  url: string;
}

interface VideoLibraryOptions {
  spaceId: string;
  signal: AbortSignal;
  sources: SourceMediaLibrary;
}

/**
 * The decodable video track and hardware playback URL of every video asset. Native codecs play
 * directly from the original file through the platform media engine; codecs the WebView cannot
 * decode are normalized by Rust into an H.264 proxy and played through the exact same hardware path.
 */
export function createVideoLibrary(options: VideoLibraryOptions) {
  const proxyInputsByKey = new Map<string, Input>();
  const converting = shallowRef<ReadonlySet<string>>(new Set());

  function disposeProxy(key: string) {
    const existing = proxyInputsByKey.get(key);
    if (!existing) return;
    proxyInputsByKey.delete(key);
    existing.dispose();
  }

  function setConverting(asset: VideoAsset, active: boolean) {
    const next = new Set(converting.value);
    if (active) next.add(assetIdentity(asset));
    else next.delete(assetIdentity(asset));
    converting.value = next;
  }

  async function proxyTrack(asset: VideoAsset): Promise<PreviewVideo> {
    setConverting(asset, true);
    try {
      const path = await prepareVideoEditProxy(mediaSourceOf(options.spaceId, asset), options.signal);
      const url = versionedMediaSource(path, asset.sourceRevision)!;
      const key = assetIdentity(asset);
      disposeProxy(key);
      const input = new Input({
        formats: ALL_FORMATS,
        source: new UrlSource(url),
      });
      proxyInputsByKey.set(key, input);
      const track = await input.getPrimaryVideoTrack();
      if (!track || !(await track.canDecode()))
        throw new Error("This video cannot be decoded on this device");
      return { track, url };
    } finally {
      setConverting(asset, false);
    }
  }

  async function open(asset: VideoAsset): Promise<PreviewVideo> {
    const source = await options.sources.load(asset);
    if (source.video) return { track: source.video, url: source.url };
    return await proxyTrack(asset);
  }

  const cache = createAssetCache(open);
  return {
    ready: cache.ready,
    load: cache.load,
    forget(asset: VideoAsset) {
      disposeProxy(assetIdentity(asset));
      return cache.forget(asset);
    },
    /** Whether the asset is waiting for a proxy because its own codec cannot be decoded here. */
    converting: (asset: VideoAsset) => converting.value.has(assetIdentity(asset)),
    dispose() {
      cache.clear();
      for (const input of proxyInputsByKey.values()) input.dispose();
      proxyInputsByKey.clear();
    },
  };
}

export type VideoLibrary = ReturnType<typeof createVideoLibrary>;
