import { ALL_FORMATS, CanvasSink, Input, UrlSource, type InputVideoTrack } from "mediabunny";
import { shallowRef } from "vue";

import { versionedMediaSource } from "../../media/media-element-source";
import { prepareVideoEditProxy } from "../prepare-video-edit-proxy";
import type { VideoAsset } from "../project";
import { assetIdentity, createAssetCache } from "./asset-cache";
import { mediaSourceOf } from "./source-ipc";
import type { SourceMediaLibrary } from "./source-media";

/**
 * Canvases the sink keeps in its ring buffer. The presentation queue holds a few frames while the
 * decoder reads ahead, so the pool keeps the allocated surfaces constant instead of growing.
 */
const POOL_SIZE = 8;
/** Largest side a preview frame is decoded to; the stage never shows more than this. */
const PREVIEW_EDGE = 1_920;

export interface PreviewVideo {
  track: InputVideoTrack;
  sink: CanvasSink;
}

/** The decoded size of a preview frame: the source's own, scaled into the preview edge. */
export function previewFrameSize(width: number, height: number) {
  const scale = Math.min(1, PREVIEW_EDGE / Math.max(1, width), PREVIEW_EDGE / Math.max(1, height));
  if (scale === 1) return {};
  return { width: Math.round(width * scale), height: Math.round(height * scale), fit: "fill" as const };
}

interface VideoLibraryOptions {
  spaceId: string;
  signal: AbortSignal;
  sources: SourceMediaLibrary;
}

/**
 * The decodable video track of every video asset. The source itself is decoded whenever the WebView
 * can, so a video is on the stage and the timeline as soon as its container index is read, as in
 * any desktop editor. Only a codec the WebView cannot decode is normalized by Rust into a proxy,
 * and that wait is published so the editor can say what it is waiting for.
 */
export function createVideoLibrary(options: VideoLibraryOptions) {
  const proxyInputs = new Set<Input>();
  const converting = shallowRef<ReadonlySet<string>>(new Set());

  function setConverting(asset: VideoAsset, active: boolean) {
    const next = new Set(converting.value);
    if (active) next.add(assetIdentity(asset));
    else next.delete(assetIdentity(asset));
    converting.value = next;
  }

  async function proxyTrack(asset: VideoAsset) {
    setConverting(asset, true);
    try {
      const path = await prepareVideoEditProxy(mediaSourceOf(options.spaceId, asset), options.signal);
      const input = new Input({
        formats: ALL_FORMATS,
        source: new UrlSource(versionedMediaSource(path, asset.sourceRevision)!),
      });
      proxyInputs.add(input);
      const track = await input.getPrimaryVideoTrack();
      if (!track || !(await track.canDecode()))
        throw new Error("This video cannot be decoded on this device");
      return track;
    } finally {
      setConverting(asset, false);
    }
  }

  async function open(asset: VideoAsset): Promise<PreviewVideo> {
    const track = (await options.sources.load(asset)).video ?? (await proxyTrack(asset));
    const size = previewFrameSize(track.displayWidth, track.displayHeight);
    return { track, sink: new CanvasSink(track, { ...size, poolSize: POOL_SIZE }) };
  }

  const cache = createAssetCache(open);
  return {
    ready: cache.ready,
    load: cache.load,
    forget: cache.forget,
    /** Whether the asset is waiting for a proxy because its own codec cannot be decoded here. */
    converting: (asset: VideoAsset) => converting.value.has(assetIdentity(asset)),
    dispose() {
      cache.clear();
      for (const input of proxyInputs) input.dispose();
      proxyInputs.clear();
    },
  };
}

export type VideoLibrary = ReturnType<typeof createVideoLibrary>;
