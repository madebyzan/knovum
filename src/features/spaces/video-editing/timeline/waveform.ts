import { shallowRef } from "vue";

import type { AudioGraph } from "../engine/audio-graph";
import type { PreviewSources } from "../engine/preview-sources";
import type { VideoMaterialNode } from "../material";
import type { VideoAsset } from "../project";

const PEAKS = 1_200;
const MAX_DECODE_MS = 15 * 60_000;

interface WaveformOptions {
  sources: PreviewSources;
  /** The preview's audio context; decoding here keeps the editor on a single context. */
  audio: AudioGraph;
}

/**
 * Peak envelopes for audio lanes. Decoding happens once per asset, one asset at a time, because a
 * waveform is a navigation aid and must never compete with playback for memory or decode time.
 */
export function createWaveformCache(options: WaveformOptions) {
  /** Published envelopes; reading this is what makes a lane repaint when decoding lands. */
  const peaks = shallowRef<ReadonlyMap<string, Float32Array>>(new Map());
  const pending = new Set<string>();

  function get(asset: VideoAsset) {
    return peaks.value.get(key(asset));
  }

  function request(asset: VideoAsset, node: VideoMaterialNode) {
    const identity = key(asset);
    if (
      asset.kind !== "audio" ||
      peaks.value.has(identity) ||
      pending.has(identity) ||
      pending.size > 0 ||
      asset.durationMs > MAX_DECODE_MS
    )
      return;
    pending.add(identity);
    void fill(identity, asset, node);
  }

  async function fill(identity: string, asset: VideoAsset, node: VideoMaterialNode) {
    let envelope = new Float32Array();
    try {
      envelope = await decode(asset, node);
    } finally {
      pending.delete(identity);
      peaks.value = new Map(peaks.value).set(identity, envelope);
    }
  }

  async function decode(asset: VideoAsset, node: VideoMaterialNode) {
    const source = options.sources.ready(asset) ?? (await options.sources.load(asset, node));
    const response = await fetch(source.url);
    return envelopeOf(await options.audio.decode(await response.arrayBuffer()));
  }

  return {
    get,
    request,
    dispose() {
      peaks.value = new Map();
      pending.clear();
    },
  };
}

function key(asset: VideoAsset) {
  return `${asset.nodeId}\u0000${asset.sourceRevision}`;
}

function envelopeOf(buffer: AudioBuffer) {
  const samples = buffer.getChannelData(0);
  const span = Math.max(1, Math.floor(samples.length / PEAKS));
  const envelope = new Float32Array(Math.ceil(samples.length / span));
  for (let index = 0; index < envelope.length; index += 1) {
    let peak = 0;
    const from = index * span;
    for (let at = from; at < Math.min(from + span, samples.length); at += 1)
      peak = Math.max(peak, Math.abs(samples[at]!));
    envelope[index] = peak;
  }
  return envelope;
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
