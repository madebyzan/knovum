import { shallowRef } from "vue";

import { waitForMediaMetadata } from "../../media/media-element-source";
import type { PreviewSources } from "../engine/preview-sources";
import type { VideoMaterialNode } from "../material";
import type { VideoAsset } from "../project";

const TILE_WIDTH = 72;
const TILE_LIMIT = 240;
/** Tiles waiting to be decoded; the queue is replaced as the viewport moves, never appended to. */
const QUEUE_LIMIT = 32;
const BUCKETS_MS = [250, 500, 1_000, 2_000, 5_000, 10_000, 30_000, 60_000];

/** Tile length that keeps filmstrip frames about one tile wide at the current scale. */
export function filmstripBucketMs(pxPerSecond: number) {
  const target = (TILE_WIDTH * 1000) / Math.max(1, pxPerSecond);
  return BUCKETS_MS.find((bucket) => bucket >= target) ?? BUCKETS_MS.at(-1)!;
}

interface FilmstripOptions {
  sources: PreviewSources;
  /** Playback owns the decoder; thumbnail extraction waits for it to stop. */
  suspended: () => boolean;
}

interface Work {
  asset: VideoAsset;
  node: VideoMaterialNode;
  bucketMs: number;
  index: number;
}

/**
 * Decodes filmstrip tiles for the visible part of the timeline. Work is bounded and replaceable, so
 * scrolling or zooming abandons tiles nobody is looking at instead of queueing a decode of every
 * frame of every clip.
 */
export function createFilmstripCache(options: FilmstripOptions) {
  const tiles = new Map<string, Map<number, ImageBitmap>>();
  const extractors = new Map<string, HTMLVideoElement>();
  /** Published snapshot of the decoded tiles; reading it is what makes a lane repaint. */
  const decoded = shallowRef<ReadonlyMap<string, ReadonlyMap<number, ImageBitmap>>>(new Map());
  let queue: Work[] = [];
  let running = false;
  let announcing = false;

  function tile(asset: VideoAsset, bucketMs: number, index: number) {
    return decoded.value.get(bucketKey(asset, bucketMs))?.get(index);
  }

  /** Replaces the outstanding work for one clip window with exactly the tiles it still needs. */
  function request(asset: VideoAsset, node: VideoMaterialNode, bucketMs: number, indices: readonly number[]) {
    if (asset.kind !== "video") return;
    const key = bucketKey(asset, bucketMs);
    const bucket = tiles.get(key) ?? new Map<number, ImageBitmap>();
    tiles.set(key, bucket);
    const wanted = indices.filter((index) => !bucket.has(index)).slice(0, QUEUE_LIMIT);
    queue = queue.filter((item) => bucketKey(item.asset, item.bucketMs) !== key);
    if (bucket.size < TILE_LIMIT) queue.push(...wanted.map((index) => ({ asset, node, bucketMs, index })));
    void pump();
  }

  async function pump() {
    if (running) return;
    running = true;
    try {
      await drain();
    } finally {
      running = false;
    }
  }

  /** One tile at a time: a timeline thumbnail must never outbid playback for the decoder. */
  async function drain(): Promise<void> {
    const work = queue.shift();
    if (!work || options.suspended()) return;
    const bucket = tiles.get(bucketKey(work.asset, work.bucketMs));
    if (bucket && !bucket.has(work.index)) {
      const bitmap = await decode(work).catch(() => undefined);
      if (bitmap) bucket.set(work.index, bitmap);
      announce();
    }
    return drain();
  }

  /** One repaint per animation frame, however many tiles land in it. */
  function announce() {
    if (announcing) return;
    announcing = true;
    requestAnimationFrame(() => {
      announcing = false;
      decoded.value = new Map(tiles);
    });
  }

  async function extractor(asset: VideoAsset, node: VideoMaterialNode) {
    const key = `${asset.nodeId}\u0000${asset.sourceRevision}`;
    const existing = extractors.get(key);
    if (existing) return existing;
    const source = options.sources.ready(asset) ?? (await options.sources.load(asset, node));
    const element = document.createElement("video");
    element.crossOrigin = "anonymous";
    element.muted = true;
    element.preload = "auto";
    element.src = source.url;
    element.load();
    await waitForMediaMetadata(element, "Timeline thumbnails are unavailable");
    if (extractors.size >= 2) {
      const [oldest] = extractors.keys();
      if (oldest) release(oldest);
    }
    extractors.set(key, element);
    return element;
  }

  async function decode(work: Work) {
    const element = await extractor(work.asset, work.node);
    const position = Math.min(work.asset.durationMs - 1, (work.index + 0.5) * work.bucketMs) / 1000;
    await seek(element, Math.max(0, position));
    const height = Math.max(
      2,
      Math.round((TILE_WIDTH * element.videoHeight) / Math.max(1, element.videoWidth)),
    );
    const canvas = document.createElement("canvas");
    canvas.width = TILE_WIDTH;
    canvas.height = height;
    const context = canvas.getContext("2d");
    if (!context) return undefined;
    context.drawImage(element, 0, 0, TILE_WIDTH, height);
    return createImageBitmap(canvas);
  }

  function release(key: string) {
    const element = extractors.get(key);
    if (!element) return;
    extractors.delete(key);
    element.removeAttribute("src");
    element.load();
  }

  return {
    tile,
    request,
    /** Called when playback stops, so queued tiles resume decoding. */
    resume: () => void pump(),
    dispose() {
      queue = [];
      decoded.value = new Map();
      for (const key of Array.from(extractors.keys())) release(key);
      for (const bucket of tiles.values()) for (const bitmap of bucket.values()) bitmap.close();
      tiles.clear();
    },
  };
}

function bucketKey(asset: VideoAsset, bucketMs: number) {
  return `${asset.nodeId}\u0000${asset.sourceRevision}\u0000${bucketMs}`;
}

function seek(element: HTMLVideoElement, position: number) {
  return new Promise<void>((resolve, reject) => {
    const done = (error?: Error) => {
      element.removeEventListener("seeked", onSeeked);
      element.removeEventListener("error", onError);
      if (error) reject(error);
      else resolve();
    };
    const onSeeked = () => done();
    const onError = () => done(new Error("Timeline thumbnail decoding failed"));
    element.addEventListener("seeked", onSeeked, { once: true });
    element.addEventListener("error", onError, { once: true });
    element.currentTime = position;
  });
}
