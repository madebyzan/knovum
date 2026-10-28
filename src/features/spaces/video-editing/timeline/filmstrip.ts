import { CanvasSink, EncodedPacketSink } from "mediabunny";
import { shallowRef } from "vue";

import { assetIdentity, createAssetCache } from "../media/asset-cache";
import type { VideoLibrary } from "../media/preview-video";
import type { VideoAsset } from "../project";
import { TIMELINE_ROWS } from "./track-geometry";

/**
 * Device pixels a thumbnail is decoded at, top to bottom: the video lane at twice its CSS height,
 * so a frame is drawn at or below its decoded size on every display Knovum supports.
 */
const THUMBNAIL_HEIGHT = TIMELINE_ROWS.video * 2;
/** Decoded thumbnails kept, least recently drawn first out: about 60 MB of 16:9 frames. */
const TILE_CAPACITY = 1_200;
/** Thumbnails one request may queue; the viewport asks again as soon as these land. */
const QUEUE_LIMIT = 64;
/**
 * Times a thumbnail can stand for. A slot shows the frame at the start of its step, so thumbnails
 * are shared between nearby zoom levels instead of being decoded again for every scale.
 */
const STEPS_MS = [40, 100, 250, 500, 1_000, 2_000, 5_000, 10_000, 15_000, 30_000, 60_000, 120_000, 300_000];

/** The largest step no longer than one slot, so neighbouring slots never show the same frame. */
export function filmstripStepMs(slotMs: number) {
  return STEPS_MS.findLast((step) => step <= slotMs) ?? STEPS_MS[0]!;
}

interface FilmstripOptions {
  videos: VideoLibrary;
  /** Playback owns the decoder; thumbnail extraction waits for it to stop. */
  suspended: () => boolean;
}

/** `exact` work replaces the key frame shown first with the frame the slot actually starts at. */
interface Work {
  asset: VideoAsset;
  stepMs: number;
  indices: number[];
  exact: boolean;
}

interface Strip {
  frames: CanvasSink;
  packets: EncodedPacketSink;
}

interface Tile {
  image: CanvasImageSource;
  exact: boolean;
}

const stepKey = (asset: VideoAsset, stepMs: number) => `${assetIdentity(asset)}\u0000${stepMs}`;
const tileKey = (asset: VideoAsset, stepMs: number, index: number) =>
  `${stepKey(asset, stepMs)}\u0000${index}`;

/** The key frame at or before a position: decoding it needs no other picture of the source. */
async function keyFrameAt(packets: EncodedPacketSink, timestampS: number) {
  const packet =
    (await packets.getKeyPacket(timestampS, { metadataOnly: true })) ??
    (await packets.getFirstKeyPacket({ metadataOnly: true }));
  return packet?.timestamp ?? timestampS;
}

/** The pictures a piece of work decodes, in decode order, and the thumbnails each one fills. */
async function decodePlan(packets: EncodedPacketSink, work: Work) {
  const timed = await Promise.all(
    work.indices.map(async (index) => {
      const startS = (index * work.stepMs) / 1000;
      return { index, at: work.exact ? startS : await keyFrameAt(packets, startS) };
    }),
  );
  const indicesByTime = new Map<number, number[]>();
  for (const { index, at } of timed) indicesByTime.set(at, [...(indicesByTime.get(at) ?? []), index]);
  return [...indicesByTime.keys()]
    .toSorted((left, right) => left - right)
    .map((at) => ({ at, indices: indicesByTime.get(at)! }));
}

/**
 * Filmstrip thumbnails, one per slot of the lane, each showing the frame its slot starts at, as
 * desktop editors draw them. A slot first shows the key frame before it, which needs one decoded
 * picture, so a strip of any source fills at once; the exact frame then replaces it. Thumbnails are
 * decoded at the lane's device-pixel height, only the slots on screen are requested, clips of one
 * source share its queue, the least recently drawn are evicted, and playback always wins the
 * decoder because a batch stops as soon as the preview starts.
 */
export function createFilmstripCache(options: FilmstripOptions) {
  /** Bumped when thumbnails land; reading it is what repaints a lane. */
  const revision = shallowRef(0);
  const tiles = new Map<string, Tile>();
  /** The step each asset was last asked at; work queued for another zoom level is stale. */
  const stepByAsset = new Map<string, number>();
  const strips = createAssetCache<Strip>(async (asset) => {
    const { track } = await options.videos.load(asset);
    return {
      frames: new CanvasSink(track, { height: THUMBNAIL_HEIGHT, poolSize: 0 }),
      packets: new EncodedPacketSink(track),
    };
  });
  let queue: Work[] = [];
  let working = false;

  function keep(key: string, tile: Tile) {
    if (tiles.get(key)?.exact && !tile.exact) return;
    tiles.delete(key);
    tiles.set(key, tile);
    while (tiles.size > TILE_CAPACITY) tiles.delete(tiles.keys().next().value!);
  }

  function enqueue(work: Work) {
    const key = stepKey(work.asset, work.stepMs);
    const queued = queue.find(
      (entry) => entry.exact === work.exact && stepKey(entry.asset, entry.stepMs) === key,
    );
    const indices = [...new Set([...work.indices, ...(queued?.indices ?? [])])].slice(0, QUEUE_LIMIT);
    queue = [...queue.filter((entry) => entry !== queued), { ...work, indices }];
  }

  async function run(work: Work) {
    if (stepByAsset.get(assetIdentity(work.asset)) !== work.stepMs) return;
    const strip = await strips.load(work.asset).catch(() => undefined);
    if (!strip) return;
    const plan = await decodePlan(strip.packets, work);
    let decoded = 0;
    for await (const frame of strip.frames.canvasesAtTimestamps(plan.map(({ at }) => at))) {
      const { indices } = plan[decoded]!;
      decoded += 1;
      if (frame) {
        for (const index of indices) {
          keep(tileKey(work.asset, work.stepMs, index), { image: frame.canvas, exact: work.exact });
        }
      }
      revision.value += 1;
      if (options.suspended()) {
        const remaining = plan.slice(decoded).flatMap((step) => step.indices);
        if (remaining.length > 0) queue.unshift({ ...work, indices: remaining });
        return;
      }
    }
    if (!work.exact) enqueue({ ...work, exact: true });
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
    tile(asset: VideoAsset, stepMs: number, index: number) {
      void revision.value;
      const key = tileKey(asset, stepMs, index);
      const tile = tiles.get(key);
      if (tile) keep(key, tile);
      return tile?.image;
    },
    /** Asks for the thumbnails of one clip's visible slots, nearest the latest request first. */
    request(asset: VideoAsset, stepMs: number, indices: number[]) {
      if (asset.kind !== "video") return;
      stepByAsset.set(assetIdentity(asset), stepMs);
      const missing = indices.filter((index) => !tiles.has(tileKey(asset, stepMs, index)));
      const inexact = indices.filter((index) => tiles.get(tileKey(asset, stepMs, index))?.exact === false);
      if (missing.length > 0) enqueue({ asset, stepMs, indices: missing, exact: false });
      if (inexact.length > 0) enqueue({ asset, stepMs, indices: inexact, exact: true });
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
