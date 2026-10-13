import type { VideoMaterialNode } from "../material";
import type { VideoAsset } from "../project";
import type { AudioGraph } from "./audio-graph";
import { canPresent, syncMedia, type MediaTarget } from "./media-sync";
import type { PreviewSource, PreviewSources } from "./preview-sources";

const POOL_LIMIT = 6;

export interface PooledMedia {
  kind: VideoAsset["kind"];
  element: HTMLVideoElement | HTMLAudioElement | HTMLImageElement;
  /** Ready to paint the position it was last synchronized to. */
  presentable: boolean;
}

interface PoolEntry extends PooledMedia {
  key: string;
  usedAt: number;
  failed: boolean;
}

interface MediaPoolOptions {
  sources: PreviewSources;
  audio: AudioGraph;
  onFailed: (error: unknown) => void;
}

/**
 * Owns the decoding media elements behind the preview. Elements live for as long as their asset is
 * near the playhead, so a cut plays an element that was already loaded and seeked instead of
 * mounting a fresh one.
 */
export function createMediaPool(options: MediaPoolOptions) {
  const entries = new Map<string, PoolEntry>();

  /** Ensures an element exists for the asset and parks it at the requested position. */
  function prepare(asset: VideoAsset, node: VideoMaterialNode, sourceMs: number) {
    const identity = key(asset);
    const existing = entries.get(identity);
    if (existing) {
      existing.usedAt = performance.now();
      if (existing.element instanceof HTMLMediaElement && existing.element.paused)
        syncMedia(existing.element, { sourceMs, playing: false });
      return existing;
    }
    const source = options.sources.ready(asset);
    if (!source) {
      void options.sources.load(asset, node).catch(options.onFailed);
      return undefined;
    }
    const element = create(source);
    const entry: PoolEntry = {
      key: identity,
      kind: asset.kind,
      element,
      presentable: false,
      usedAt: performance.now(),
      failed: false,
    };
    element.addEventListener("error", () => {
      entry.failed = true;
      options.onFailed(new Error("Preview media is unavailable"));
    });
    if (element instanceof HTMLMediaElement) {
      element.addEventListener("loadedmetadata", () => syncMedia(element, { sourceMs, playing: false }), {
        once: true,
      });
      options.audio.setGain(element, 0);
    } else {
      element.addEventListener("load", () => (entry.presentable = true), { once: true });
    }
    entries.set(identity, entry);
    evict();
    return entry;
  }

  /** Per-frame correction for one active layer. */
  function sync(entry: PooledMedia, target: MediaTarget & { gain: number }) {
    const element = entry.element;
    if (!(element instanceof HTMLMediaElement)) {
      entry.presentable = element.complete && element.naturalWidth > 0;
      return entry.presentable;
    }
    options.audio.setGain(element, target.gain);
    const result = syncMedia(element, target);
    if (result !== "parked" && target.playing && element.paused) void start(element);
    entry.presentable = canPresent(element);
    return result === "playing" || (!target.playing && entry.presentable);
  }

  async function start(element: HTMLMediaElement) {
    try {
      await options.audio.resume();
      await element.play();
    } catch (error) {
      options.onFailed(error);
    }
  }

  function pauseAll() {
    for (const entry of entries.values())
      if (entry.element instanceof HTMLMediaElement) entry.element.pause();
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
      entry.element.pause();
      entry.element.removeAttribute("src");
      entry.element.load();
    } else entry.element.removeAttribute("src");
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
