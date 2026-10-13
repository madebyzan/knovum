import { ALL_FORMATS, CanvasSink, Input, UrlSource, type InputVideoTrack } from "mediabunny";

import { versionedMediaSource } from "../../media/media-element-source";
import { prepareVideoEditProxy } from "../prepare-video-edit-proxy";
import type { VideoAsset } from "../project";
import { createAssetCache } from "./asset-cache";

/**
 * Canvases the sink keeps in its ring buffer. The presentation queue holds a few frames while the
 * decoder reads ahead, so the pool keeps the allocated surfaces constant instead of growing.
 */
const POOL_SIZE = 8;

export interface PreviewVideo {
  url: string;
  track: InputVideoTrack;
  sink: CanvasSink;
}

/**
 * Opens the preview media of every video asset once per revision. Rust owns normalization: it hands
 * back a short-GOP H.264 file, and mediabunny owns demuxing and hardware-accelerated decoding of
 * it, so the editor never drives a media element or guesses a seek.
 */
export function createVideoLibrary(spaceId: string, signal: AbortSignal) {
  const inputs = new Set<Input>();

  async function open(asset: VideoAsset): Promise<PreviewVideo> {
    const path = await prepareVideoEditProxy(
      { spaceId, sourceNodeId: asset.nodeId, sourceRevision: asset.sourceRevision },
      signal,
    );
    const url = versionedMediaSource(path, asset.sourceRevision)!;
    const input = new Input({ formats: ALL_FORMATS, source: new UrlSource(url) });
    inputs.add(input);
    const track = await input.getPrimaryVideoTrack();
    if (!track) throw new Error("Prepared preview media carries no video track");
    if (!(await track.canDecode())) throw new Error("This video cannot be decoded on this device");
    return { url, track, sink: new CanvasSink(track, { poolSize: POOL_SIZE }) };
  }

  const cache = createAssetCache(open);
  return {
    ready: cache.ready,
    load: cache.load,
    forget: cache.forget,
    dispose() {
      cache.clear();
      for (const input of inputs) input.dispose();
      inputs.clear();
    },
  };
}

export type VideoLibrary = ReturnType<typeof createVideoLibrary>;
