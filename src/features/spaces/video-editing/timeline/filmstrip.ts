import { CanvasSink, EncodedPacketSink } from "mediabunny";
import { shallowRef } from "vue";

import { assetIdentity, createAssetCache } from "../media/asset-cache";
import type { VideoLibrary } from "../media/preview-video";
import type { VideoAsset } from "../project";

const TILE_WIDTH = 128;
/** Decoded tiles kept, least recently drawn first out: about 30 MB of thumbnails. */
const TILE_CAPACITY = 900;
/** Tiles one request may queue; the viewport asks again as soon as these land. */
const QUEUE_LIMIT = 48;
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

interface Strip {
  frames: CanvasSink;
  packets: EncodedPacketSink;
}

const bucketKey = (asset: VideoAsset, bucketMs: number) => `${assetIdentity(asset)}\u0000${bucketMs}`;
const tileKey = (asset: VideoAsset, bucketMs: number, index: number) =>
  `${bucketKey(asset, bucketMs)}\u0000${index}`;

/** The key frame at or before a position: decoding it needs no other picture of the source. */
async function keyFrameAt(packets: EncodedPacketSink, timestampS: number) {
  const packet =
    (await packets.getKeyPacket(timestampS, { metadataOnly: true })) ??
    (await packets.getFirstKeyPacket({ metadataOnly: true }));
  return packet?.timestamp ?? timestampS;
}

/** The tiles of a request, grouped by the key frame each one shows, in decode order. */
async function groupByKeyFrame(packets: EncodedPacketSink, work: Work) {
  const keyed = await Promise.all(
    work.indices.map(async (index) => ({
      index,
      at: await keyFrameAt(packets, (index * work.bucketMs) / 1000),
    })),
  );
  const indicesByKeyFrame = new Map<number, number[]>();
  for (const { index, at } of keyed) indicesByKeyFrame.set(at, [...(indicesByKeyFrame.get(at) ?? []), index]);
  const keyFrames = [...indicesByKeyFrame.keys()].toSorted((left, right) => left - right);
  return keyFrames.map((at) => ({ at, indices: indicesByKeyFrame.get(at)! }));
}

/**
 * Filmstrip tiles. A tile shows the key frame nearest before it, as desktop editors' thumbnails do:
 * each costs one decoded picture however far apart key frames are, so a long source fills its
 * strip in moments instead of decoding through every group of pictures. Only tiles the viewport
 * asks for are decoded, and playback always wins the decoder because a batch stops as soon as the
 * preview starts.
 */
export function createFilmstripCache(options: FilmstripOptions) {
  /** Bumped when tiles land; reading it is what repaints a lane. */
  const revision = shallowRef(0);
  const tiles = new Map<string, CanvasImageSource>();
  /** The scale each asset was last asked at; work queued for another scale is stale. */
  const bucketByAsset = new Map<string, number>();
  const strips = createAssetCache<Strip>(async (asset) => {
    const { track } = await options.videos.load(asset);
    return {
      frames: new CanvasSink(track, { width: TILE_WIDTH, poolSize: 0 }),
      packets: new EncodedPacketSink(track),
    };
  });
  let queue: Work[] = [];
  let working = false;

  function keep(key: string, tile: CanvasImageSource) {
    tiles.delete(key);
    tiles.set(key, tile);
    while (tiles.size > TILE_CAPACITY) tiles.delete(tiles.keys().next().value!);
  }

  async function run(work: Work) {
    if (bucketByAsset.get(assetIdentity(work.asset)) !== work.bucketMs) return;
    const strip = await strips.load(work.asset).catch(() => undefined);
    if (!strip) return;
    const groups = await groupByKeyFrame(strip.packets, work);
    let decoded = 0;
    for await (const frame of strip.frames.canvasesAtTimestamps(groups.map(({ at }) => at))) {
      const { indices } = groups[decoded]!;
      decoded += 1;
      if (frame) for (const index of indices) keep(tileKey(work.asset, work.bucketMs, index), frame.canvas);
      revision.value += 1;
      if (options.suspended()) {
        const remaining = groups.slice(decoded).flatMap((group) => group.indices);
        if (remaining.length > 0) queue.unshift({ ...work, indices: remaining });
        break;
      }
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
    tile(asset: VideoAsset, bucketMs: number, index: number) {
      void revision.value;
      const key = tileKey(asset, bucketMs, index);
      const tile = tiles.get(key);
      if (tile) keep(key, tile);
      return tile;
    },
    /**
     * Asks for tiles of one clip's visible window. Clips of the same source share its queue entry,
     * so two clips on screen both fill in, nearest the latest request first.
     */
    request(asset: VideoAsset, bucketMs: number, indices: number[]) {
      if (asset.kind !== "video") return;
      bucketByAsset.set(assetIdentity(asset), bucketMs);
      const missing = indices.filter((index) => !tiles.has(tileKey(asset, bucketMs, index)));
      if (missing.length === 0) return;
      const key = bucketKey(asset, bucketMs);
      const queued = queue.find((work) => bucketKey(work.asset, work.bucketMs) === key);
      const merged = [...new Set([...missing, ...(queued?.indices ?? [])])].slice(0, QUEUE_LIMIT);
      queue = [...queue.filter((work) => work !== queued), { asset, bucketMs, indices: merged }];
      start();
    },
    /** Playback stopped: pick the queue back up. */
    resume: start,
    dispose() {
      queue = [];
      strips.clear();
      tiles.clear();
      revision.value += 1;
    },
  };
}
