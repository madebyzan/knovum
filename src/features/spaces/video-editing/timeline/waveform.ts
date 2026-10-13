import { shallowRef } from "vue";

import { assetIdentity } from "../media/asset-cache";
import { computePeaks } from "../media/audio-peaks";
import type { AudioLibrary } from "../media/preview-audio";
import type { VideoAsset } from "../project";

const PEAKS = 1_200;

interface WaveformOptions {
  /** The preview's decoded audio; a waveform never decodes a second copy of a source. */
  library: AudioLibrary;
}

/**
 * Peak envelopes for audio lanes, reduced from the buffers the preview already decoded for
 * playback. A waveform is a navigation aid, so it never triggers its own decode pass.
 */
export function createWaveformCache(options: WaveformOptions) {
  /** Published envelopes; reading this is what makes a lane repaint when peaks land. */
  const peaks = shallowRef<ReadonlyMap<string, Float32Array>>(new Map());
  const pending = new Set<string>();

  function publish(identity: string, envelope: Float32Array) {
    pending.delete(identity);
    peaks.value = new Map(peaks.value).set(identity, envelope);
  }

  return {
    get: (asset: VideoAsset) => peaks.value.get(assetIdentity(asset)),
    request(asset: VideoAsset) {
      const identity = assetIdentity(asset);
      if (asset.kind === "image" || peaks.value.has(identity) || pending.has(identity)) return;
      pending.add(identity);
      void options.library.load(asset).then(
        (buffer) => publish(identity, buffer ? computePeaks(buffer, PEAKS) : new Float32Array()),
        () => publish(identity, new Float32Array()),
      );
    },
    dispose() {
      peaks.value = new Map();
      pending.clear();
    },
  };
}

/** Resamples an envelope to the bars one clip can show for its own source window. */
export function waveformBars(
  envelope: Float32Array | number[] | undefined,
  window: { fromRatio: number; toRatio: number },
  bars: number,
) {
  if (!envelope || envelope.length === 0 || bars <= 0) return [];
  const from = Math.floor(window.fromRatio * envelope.length);
  const to = Math.max(from + 1, Math.ceil(window.toRatio * envelope.length));
  const span = (to - from) / bars;
  return Array.from({ length: bars }, (_, index) => {
    const start = Math.floor(from + index * span);
    const end = Math.max(start + 1, Math.floor(from + (index + 1) * span));
    let peak = 0;
    for (let at = start; at < Math.min(end, envelope.length); at += 1)
      peak = Math.max(peak, envelope[at] ?? 0);
    return peak;
  });
}
