import type { PreviewSources } from "../engine/preview-sources";
import type { VideoMaterialNode } from "../material";
import type { VideoAsset } from "../project";

const PEAKS = 1_200;
const MAX_DECODE_MS = 15 * 60_000;

/**
 * Peak envelopes for audio lanes. Decoding happens once per asset and is capped, because a waveform
 * is a navigation aid and must never compete with playback for memory.
 */
export function createWaveformCache(options: { sources: PreviewSources; onReady: () => void }) {
  const peaks = new Map<string, Float32Array>();
  const pending = new Set<string>();
  let context: AudioContext | undefined;

  function get(asset: VideoAsset) {
    return peaks.get(key(asset));
  }

  function request(asset: VideoAsset, node: VideoMaterialNode) {
    const identity = key(asset);
    if (
      asset.kind !== "audio" ||
      peaks.has(identity) ||
      pending.has(identity) ||
      asset.durationMs > MAX_DECODE_MS
    )
      return;
    pending.add(identity);
    void fill(identity, asset, node);
  }

  async function fill(identity: string, asset: VideoAsset, node: VideoMaterialNode) {
    try {
      peaks.set(identity, await decode(asset, node));
    } catch {
      peaks.set(identity, new Float32Array());
    } finally {
      pending.delete(identity);
      options.onReady();
    }
  }

  async function decode(asset: VideoAsset, node: VideoMaterialNode) {
    const source = options.sources.ready(asset) ?? (await options.sources.load(asset, node));
    const response = await fetch(source.url);
    const bytes = await response.arrayBuffer();
    context ??= new AudioContext();
    const buffer = await context.decodeAudioData(bytes);
    return envelopeOf(buffer);
  }

  return {
    get,
    request,
    dispose() {
      peaks.clear();
      void context?.close();
      context = undefined;
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
