import { CanvasSink } from "mediabunny";
import { shallowRef } from "vue";

import { assetIdentity } from "../media/asset-cache";
import type { VideoLibrary } from "../media/preview-video";
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
  videos: VideoLibrary;
  /** Playback owns the decoder; thumbnail extraction waits for it to stop. */
  suspended: () => boolean;
}

interface Work {
  asset: VideoAsset;
  bucketMs: number;
  indices: number[];
}

function bucketKey(asset: VideoAsset, bucketMs: number) {
  return `${assetIdentity(asset)}\u0000${bucketMs}`;
}

function tileKey(asset: VideoAsset, bucketMs: number, index: number) {
  return `${bucketKey(asset, bucketMs)}\u0000${index}`;
}

/**
 * Filmstrip tiles. Mediabunny decodes a sorted batch of timestamps in one pass, reading each packet
 * at most once; only the tiles the viewport asks for are decoded, and playback always wins the
 * decoder because a batch stops as soon as the preview starts.
 */
export function createFilmstripCache(options: FilmstripOptions) {
  /** Published tiles; reading this is what repaints a lane when a batch lands. */
  const tiles = shallowRef<ReadonlyMap<string, CanvasImageSource>>(new Map());
  const sinks = new Map<string, CanvasSink>();
  let queue: Work[] = [];
  let working = false;

  function sinkOf(asset: VideoAsset) {
    const known = sinks.get(assetIdentity(asset));
    if (known) return known;
    const preview = options.videos.ready(asset);
    if (!preview) return undefined;
    const sink = new CanvasSink(preview.track, { width: TILE_WIDTH, poolSize: 0 });
    sinks.set(assetIdentity(asset), sink);
    return sink;
  }

  async function run(work: Work) {
    const sink = sinkOf(work.asset);
    if (!sink) return;
    const indices = work.indices.toSorted((left, right) => left - right);
    const timestamps = indices.map((index) => (index * work.bucketMs) / 1000);
    let at = 0;
    for await (const frame of sink.canvasesAtTimestamps(timestamps)) {
      const index = indices[at];
      at += 1;
      if (frame && index !== undefined) {
        tiles.value = new Map(tiles.value).set(tileKey(work.asset, work.bucketMs, index), frame.canvas);
      }
      if (options.suspended()) break;
    }
  }

  async function drain() {
    const work = queue.shift();
    if (!work || options.suspended()) {
      working = false;
      return;
    }
    await run(work).catch(() => undefined);
    await drain();
  }

  function start() {
    if (working || options.suspended() || queue.length === 0) return;
    working = true;
    void drain();
  }

  return {
    tile: (asset: VideoAsset, bucketMs: number, index: number) =>
      tiles.value.get(tileKey(asset, bucketMs, index)),
    request(asset: VideoAsset, bucketMs: number, indices: number[]) {
      if (asset.kind !== "video" || tiles.value.size > TILE_LIMIT) return;
      const missing = indices.filter((index) => !tiles.value.has(tileKey(asset, bucketMs, index)));
      const key = bucketKey(asset, bucketMs);
      queue = queue.filter((work) => bucketKey(work.asset, work.bucketMs) !== key);
      if (missing.length > 0) queue.push({ asset, bucketMs, indices: missing.slice(0, QUEUE_LIMIT) });
      start();
    },
    /** Playback stopped: pick the queue back up. */
    resume: start,
    dispose() {
      queue = [];
      sinks.clear();
      tiles.value = new Map();
    },
  };
}
