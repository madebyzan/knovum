import pLimit from "p-limit";

import { waitForMediaMetadata } from "../../media/media-element-source";
import type { PreviewSources } from "../engine/preview-sources";
import type { VideoMaterialNode } from "../material";
import type { VideoAsset } from "../project";

const TILE_WIDTH = 72;
const TILE_LIMIT = 400;
const BUCKETS_MS = [250, 500, 1_000, 2_000, 5_000, 10_000, 30_000, 60_000];

/** Tile length that keeps filmstrip frames about one tile wide at the current scale. */
export function filmstripBucketMs(pxPerSecond: number) {
  const target = (TILE_WIDTH * 1000) / Math.max(1, pxPerSecond);
  return BUCKETS_MS.find((bucket) => bucket >= target) ?? BUCKETS_MS.at(-1)!;
}

interface FilmstripOptions {
  sources: PreviewSources;
  onReady: () => void;
}

/**
 * Decodes filmstrip tiles for timeline clips. Tiles are keyed by asset and tile length, so zooming
 * reuses what is already decoded, and extraction is serialized to keep decoding away from playback.
 */
export function createFilmstripCache(options: FilmstripOptions) {
  const tiles = new Map<string, Map<number, ImageBitmap>>();
  const pending = new Set<string>();
  const extractors = new Map<string, HTMLVideoElement>();
  const limit = pLimit(1);

  function tile(asset: VideoAsset, bucketMs: number, index: number) {
    return tiles.get(bucketKey(asset, bucketMs))?.get(index);
  }

  function request(asset: VideoAsset, node: VideoMaterialNode, bucketMs: number, indices: readonly number[]) {
    if (asset.kind !== "video") return;
    const key = bucketKey(asset, bucketMs);
    const existing = tiles.get(key) ?? new Map<number, ImageBitmap>();
    tiles.set(key, existing);
    for (const index of indices) {
      const identity = `${key}\u0000${index}`;
      if (existing.has(index) || pending.has(identity) || existing.size >= TILE_LIMIT) continue;
      pending.add(identity);
      void fill(identity, existing, index, () => decode(asset, node, bucketMs, index));
    }
  }

  async function fill(
    identity: string,
    bucket: Map<number, ImageBitmap>,
    index: number,
    task: () => Promise<ImageBitmap | undefined>,
  ) {
    try {
      const bitmap = await limit(task);
      if (bitmap) bucket.set(index, bitmap);
    } finally {
      pending.delete(identity);
      options.onReady();
    }
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

  async function decode(asset: VideoAsset, node: VideoMaterialNode, bucketMs: number, index: number) {
    const element = await extractor(asset, node);
    const position = Math.min(asset.durationMs - 1, (index + 0.5) * bucketMs) / 1000;
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
    dispose() {
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
