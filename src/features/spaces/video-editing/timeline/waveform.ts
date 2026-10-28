import { shallowRef } from "vue";

import type { VideoEditWaveformChunk } from "../../contracts/generated/VideoEditWaveformChunk";
import { assetIdentity } from "../media/asset-cache";
import { mediaSourceOf, streamSourceWaveform } from "../media/source-ipc";
import type { VideoAsset } from "../project";

/** Peaks of one source at a fixed rate; the part not decoded yet reads as silence. */
interface Waveform {
  rate: number;
  peaks: Uint8Array;
}

interface WaveformOptions {
  spaceId: string;
  audible: (asset: VideoAsset) => Promise<boolean>;
}

function written(waveform: Waveform | undefined, asset: VideoAsset, chunk: VideoEditWaveformChunk): Waveform {
  const needed = chunk.offset + chunk.peaks.length;
  const expected = Math.ceil((asset.durationMs * chunk.rate) / 1000);
  const peaks =
    waveform && waveform.peaks.length >= needed ? waveform.peaks : new Uint8Array(Math.max(needed, expected));
  if (waveform && peaks !== waveform.peaks) peaks.set(waveform.peaks);
  peaks.set(chunk.peaks, chunk.offset);
  return { rate: chunk.rate, peaks };
}

/**
 * Waveforms for audio lanes. Rust decodes a source's sound once at a low rate and streams its
 * peaks as it goes, so a lane fills in from the left within moments of the source being added and
 * is kept per revision for the next time. A waveform is a navigation aid: one that cannot be read
 * leaves the lane plain rather than interrupting the edit.
 */
export function createWaveformCache(options: WaveformOptions) {
  /** Bumped when peaks land; reading it is what repaints a lane. */
  const revision = shallowRef(0);
  const waveforms = new Map<string, Waveform>();
  const requested = new Set<string>();
  let disposed = false;

  async function load(asset: VideoAsset) {
    if (!(await options.audible(asset))) return;
    const identity = assetIdentity(asset);
    await streamSourceWaveform(mediaSourceOf(options.spaceId, asset), (chunk) => {
      if (disposed) return;
      waveforms.set(identity, written(waveforms.get(identity), asset, chunk));
      revision.value += 1;
    });
  }

  return {
    get(asset: VideoAsset) {
      void revision.value;
      return waveforms.get(assetIdentity(asset));
    },
    request(asset: VideoAsset) {
      const identity = assetIdentity(asset);
      if (asset.kind === "image" || requested.has(identity)) return;
      requested.add(identity);
      void load(asset).catch(() => undefined);
    },
    dispose() {
      disposed = true;
      waveforms.clear();
      requested.clear();
    },
  };
}

/** The loudest peak of each of `bars` equal slices of a source window, from 0 to 1. */
export function waveformBars(
  waveform: Waveform | undefined,
  window: { fromMs: number; toMs: number },
  bars: number,
) {
  if (!waveform || bars <= 0 || window.toMs <= window.fromMs) return [];
  const from = (window.fromMs * waveform.rate) / 1000;
  const span = ((window.toMs - window.fromMs) * waveform.rate) / 1000 / bars;
  return Array.from({ length: bars }, (_, bar) => {
    const start = Math.floor(from + bar * span);
    const end = Math.min(waveform.peaks.length, Math.max(start + 1, Math.floor(from + (bar + 1) * span)));
    let peak = 0;
    for (let at = start; at < end; at += 1) peak = Math.max(peak, waveform.peaks[at]!);
    return peak / 255;
  });
}
