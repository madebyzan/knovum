import type { VideoAsset } from "../project";
import { assetIdentity } from "./asset-cache";
import { mediaSourceOf, readSourceAudio } from "./source-ipc";

/** Length of one decoded piece of source audio; the scheduler plays pieces back to back. */
export const AUDIO_CHUNK_MS = 5_000;
/** Decoded pieces kept, least recently played first out: about three minutes of stereo audio. */
const CAPACITY = 36;
/** Pieces decoded at once; more would only compete with the piece the playhead needs next. */
const CONCURRENCY = 2;
const CHANNELS = 2;

interface AudioChunkOptions {
  spaceId: string;
  context: () => AudioContext;
  audible: (asset: VideoAsset) => Promise<boolean>;
  onFailed: (error: unknown) => void;
}

const keyOf = (asset: VideoAsset, index: number) => `${assetIdentity(asset)}\u0000${index}`;

/** De-interleaves stereo `f32le` into a buffer at the context's rate; nothing when it is empty. */
export function audioBufferOf(context: AudioContext, bytes: ArrayBuffer) {
  const samples = new Float32Array(bytes);
  const frames = Math.floor(samples.length / CHANNELS);
  if (frames === 0) return undefined;
  const buffer = context.createBuffer(CHANNELS, frames, context.sampleRate);
  for (let channel = 0; channel < CHANNELS; channel += 1) {
    const data = new Float32Array(frames);
    for (let frame = 0; frame < frames; frame += 1) data[frame] = samples[frame * CHANNELS + channel]!;
    buffer.copyToChannel(data, channel);
  }
  return buffer;
}

/**
 * Source audio in fixed pieces, decoded on demand around the playhead. A source of any length
 * starts sounding as soon as its first piece is read, and memory holds the pieces near the
 * playhead rather than the whole decoded source.
 */
export function createAudioChunks(options: AudioChunkOptions) {
  /** `null` marks a piece past the end of the audio: known, and silent. */
  const ready = new Map<string, AudioBuffer | null>();
  const pending = new Set<string>();
  /** Assets without sound, and assets whose sound could not be read: neither is asked again. */
  const silent = new Set<string>();
  const failed = new Set<string>();
  const queue: { asset: VideoAsset; index: number }[] = [];
  let active = 0;
  let disposed = false;

  function keep(key: string, buffer: AudioBuffer | null) {
    ready.delete(key);
    ready.set(key, buffer);
    while (ready.size > CAPACITY) ready.delete(ready.keys().next().value!);
  }

  async function decode(asset: VideoAsset, index: number) {
    if (!(await options.audible(asset))) {
      silent.add(assetIdentity(asset));
      return;
    }
    const fromMs = index * AUDIO_CHUNK_MS;
    const context = options.context();
    const bytes = await readSourceAudio(mediaSourceOf(options.spaceId, asset), {
      fromMs,
      lengthMs: Math.min(AUDIO_CHUNK_MS, asset.durationMs - fromMs),
      sampleRate: context.sampleRate,
    });
    if (!disposed) keep(keyOf(asset, index), audioBufferOf(context, bytes) ?? null);
  }

  function drain() {
    while (active < CONCURRENCY && queue.length > 0) {
      const { asset, index } = queue.shift()!;
      active += 1;
      void decode(asset, index)
        .catch((error: unknown) => {
          if (failed.has(assetIdentity(asset))) return;
          failed.add(assetIdentity(asset));
          options.onFailed(error);
        })
        .finally(() => {
          pending.delete(keyOf(asset, index));
          active -= 1;
          drain();
        });
    }
  }

  return {
    chunkMs: AUDIO_CHUNK_MS,
    /** The decoded piece, when it is ready; reading it keeps it from being evicted. */
    chunk(asset: VideoAsset, index: number) {
      const key = keyOf(asset, index);
      const buffer = ready.get(key);
      if (buffer) keep(key, buffer);
      return buffer ?? undefined;
    },
    request(asset: VideoAsset, index: number) {
      const key = keyOf(asset, index);
      if (index < 0 || index * AUDIO_CHUNK_MS >= asset.durationMs) return;
      const identity = assetIdentity(asset);
      if (disposed || ready.has(key) || pending.has(key) || silent.has(identity) || failed.has(identity))
        return;
      pending.add(key);
      queue.push({ asset, index });
      drain();
    },
    /** The playhead jumped: pieces queued for the old position would only delay the new one. */
    retarget() {
      for (const { asset, index } of queue.splice(0)) pending.delete(keyOf(asset, index));
    },
    forget(asset: VideoAsset) {
      const prefix = `${assetIdentity(asset)}\u0000`;
      for (const key of ready.keys()) if (key.startsWith(prefix)) ready.delete(key);
      silent.delete(assetIdentity(asset));
      failed.delete(assetIdentity(asset));
    },
    /** Reads that failed are tried again. */
    retry: () => failed.clear(),
    dispose() {
      disposed = true;
      queue.length = 0;
      ready.clear();
    },
  };
}

export type AudioChunks = ReturnType<typeof createAudioChunks>;
