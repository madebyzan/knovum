import type { VideoMaterialNode } from "../material";
import type { VideoAsset } from "../project";
import type { AudioGraph } from "./audio-graph";
import { canPresent, syncParked, syncPlaying, type MediaSyncResult } from "./media-sync";
import type { PreviewSource, PreviewSources } from "./preview-sources";

const POOL_LIMIT = 6;
/** Media events that change what a frame can show, so the preview repaints on fact, not on a poll. */
const SIGNALS = ["loadeddata", "seeked", "canplay", "playing", "waiting", "stalled"] as const;

export interface PooledMedia {
  kind: VideoAsset["kind"];
  element: HTMLVideoElement | HTMLAudioElement | HTMLImageElement;
  /** Ready to paint the position it was last synchronized to. */
  presentable: boolean;
}

interface PoolEntry extends PooledMedia {
  key: string;
  usedAt: number;
  /** A play() call is in flight; pausing or seeking now would abort it. */
  starting: boolean;
}

interface MediaPoolOptions {
  sources: PreviewSources;
  audio: AudioGraph;
  onChanged: () => void;
  onFailed: (error: unknown) => void;
}

export interface MediaRequest {
  sourceMs: number;
  playing: boolean;
  gain: number;
}

/**
 * Owns the decoding elements behind the preview. One element per asset lives for as long as the
 * asset is near the playhead, and its transport is driven only from here, so nothing pauses an
 * element the clock wants playing.
 */
export function createMediaPool(options: MediaPoolOptions) {
  const entries = new Map<string, PoolEntry>();

  /** Ensures an element exists for the asset without touching its transport. */
  function prepare(asset: VideoAsset, node: VideoMaterialNode) {
    const identity = key(asset);
    const existing = entries.get(identity);
    if (existing) {
      existing.usedAt = performance.now();
      return existing;
    }
    const source = options.sources.ready(asset);
    if (!source) {
      void options.sources.load(asset, node).then(options.onChanged).catch(options.onFailed);
      return undefined;
    }
    const entry = admit(identity, asset.kind, create(source));
    evict();
    return entry;
  }

  function admit(identity: string, kind: VideoAsset["kind"], element: PoolEntry["element"]) {
    const entry: PoolEntry = {
      key: identity,
      kind,
      element,
      presentable: false,
      usedAt: performance.now(),
      starting: false,
    };
    element.addEventListener("error", () => options.onFailed(new Error("Preview media is unavailable")));
    if (element instanceof HTMLMediaElement) {
      for (const signal of SIGNALS) element.addEventListener(signal, options.onChanged);
      options.audio.setGain(element, 0);
    } else void decodeImage(element, entry);
    entries.set(identity, entry);
    return entry;
  }

  async function decodeImage(element: HTMLImageElement, entry: PoolEntry) {
    try {
      await element.decode();
      entry.presentable = true;
    } catch {
      entry.presentable = false;
    }
    options.onChanged();
  }

  /** Per-frame correction for one layer; returns whether it can be shown at this position. */
  function sync(entry: PooledMedia, request: MediaRequest) {
    const element = entry.element;
    if (!(element instanceof HTMLMediaElement)) return entry.presentable;
    const pooled = entry as PoolEntry;
    options.audio.setGain(element, request.gain);
    const result = transport(pooled, element, request);
    entry.presentable = canPresent(element);
    return result !== "waiting";
  }

  function transport(entry: PoolEntry, element: HTMLMediaElement, request: MediaRequest): MediaSyncResult {
    if (!request.playing) {
      if (entry.starting) return "waiting";
      if (!element.paused) element.pause();
      return syncParked(element, request.sourceMs);
    }
    if (element.paused && !entry.starting) {
      entry.starting = true;
      void start(entry, element);
    }
    return syncPlaying(element, request.sourceMs);
  }

  async function start(entry: PoolEntry, element: HTMLMediaElement) {
    try {
      await options.audio.resume();
      await element.play();
    } catch (error) {
      options.onFailed(error);
    } finally {
      entry.starting = false;
      options.onChanged();
    }
  }

  function pauseAll() {
    for (const entry of entries.values())
      if (entry.element instanceof HTMLMediaElement) {
        entry.element.pause();
        entry.element.playbackRate = 1;
      }
  }

  function evict() {
    if (entries.size <= POOL_LIMIT) return;
    const ordered = [...entries.values()].toSorted((left, right) => left.usedAt - right.usedAt);
    for (const entry of ordered.slice(0, entries.size - POOL_LIMIT)) release(entry.key);
  }

  function release(identity: string) {
    const entry = entries.get(identity);
    if (!entry) return;
    entries.delete(identity);
    if (entry.element instanceof HTMLMediaElement) {
      for (const signal of SIGNALS) entry.element.removeEventListener(signal, options.onChanged);
      entry.element.pause();
      options.audio.detach(entry.element);
    }
    entry.element.removeAttribute("src");
    if (entry.element instanceof HTMLMediaElement) entry.element.load();
  }

  return {
    prepare,
    sync,
    pauseAll,
    get: (asset: VideoAsset) => entries.get(key(asset)),
    retain(keys: Set<string>) {
      for (const identity of Array.from(entries.keys())) if (!keys.has(identity)) release(identity);
    },
    identity: key,
    dispose() {
      for (const identity of Array.from(entries.keys())) release(identity);
    },
  };
}

function key(asset: VideoAsset) {
  return `${asset.nodeId}\u0000${asset.sourceRevision}`;
}

function create(source: PreviewSource) {
  if (source.kind === "image") {
    const image = new Image();
    image.decoding = "async";
    image.src = source.url;
    return image;
  }
  const element = document.createElement(source.kind === "video" ? "video" : "audio");
  element.crossOrigin = "anonymous";
  element.preload = "auto";
  if (element instanceof HTMLVideoElement) element.playsInline = true;
  element.src = source.url;
  element.load();
  return element;
}

export type MediaPool = ReturnType<typeof createMediaPool>;
