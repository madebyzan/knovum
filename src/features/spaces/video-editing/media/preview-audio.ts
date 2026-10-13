import { ALL_FORMATS, BufferTarget, Conversion, Input, Mp4OutputFormat, Output, UrlSource } from "mediabunny";

import { versionedMediaSource } from "../../media/media-element-source";
import type { VideoMaterialNode } from "../material";
import type { VideoAsset } from "../project";
import { createAssetCache } from "./asset-cache";
import type { VideoLibrary } from "./preview-video";

interface AudioLibraryOptions {
  context: () => AudioContext;
  videos: VideoLibrary;
  materials: () => ReadonlyMap<string, VideoMaterialNode>;
}

/**
 * Copies an asset's audio track out of its preview file without re-encoding and decodes the result
 * to PCM. Only the audio bytes are read, and decoding stays on the Web Audio decoder that every
 * supported target implements rather than on a WebCodecs audio decoder.
 */
async function extractTrackAudio(url: string, context: AudioContext, signal?: AbortSignal) {
  try {
    const input = new Input({ formats: ALL_FORMATS, source: new UrlSource(url) });
    try {
      const target = new BufferTarget();
      const conversion = await Conversion.init({
        input,
        output: new Output({ format: new Mp4OutputFormat(), target }),
        video: { discard: true },
      });
      if (conversion.isValid) {
        await conversion.execute();
        if (target.buffer && target.buffer.byteLength > 0) {
          const pcm = await context.decodeAudioData(target.buffer.slice(0));
          if (pcm) return pcm;
        }
      }
    } finally {
      input.dispose();
    }
  } catch {
    // Fall back to direct file fetch and decode when demuxing is unavailable
  }
  try {
    const response = await fetch(url, signal ? { signal } : {});
    if (!response.ok) return undefined;
    return await context.decodeAudioData(await response.arrayBuffer());
  } catch {
    return undefined;
  }
}

async function decodeManagedFile(url: string, context: AudioContext, signal: AbortSignal) {
  const response = await fetch(url, { signal });
  if (!response.ok) throw new Error("Cannot read the selected audio");
  return context.decodeAudioData(await response.arrayBuffer());
}

/**
 * Keeps one decoded buffer per audio-bearing asset. Scheduled playback needs the samples up front,
 * and the waveform reads its peaks from the same buffer instead of decoding a second time.
 */
export function createAudioLibrary(options: AudioLibraryOptions) {
  const controller = new AbortController();

  async function open(asset: VideoAsset) {
    if (asset.kind === "image") return undefined;
    if (asset.kind === "video") {
      const preview = await options.videos.load(asset);
      return extractTrackAudio(preview.url, options.context(), controller.signal);
    }
    const node = options.materials().get(asset.nodeId);
    const url = versionedMediaSource(node?.data.assetPath, asset.sourceRevision);
    return url ? decodeManagedFile(url, options.context(), controller.signal) : undefined;
  }

  const cache = createAssetCache(open);
  return {
    ready: cache.ready,
    loaded: cache.loaded,
    load: cache.load,
    forget: cache.forget,
    dispose() {
      controller.abort();
      cache.clear();
    },
  };
}

export type AudioLibrary = ReturnType<typeof createAudioLibrary>;
