import { describe, expect, it, vi } from "vitest";

import { createFrameResolver } from "../engine/frame-resolver";
import type { PooledMedia } from "../engine/media-pool";
import { syncParked, syncPlaying } from "../engine/media-sync";
import type { VideoAsset, VideoClip } from "../project";

interface FakeMedia {
  readyState: number;
  seeking: boolean;
  paused: boolean;
  currentTime: number;
  playbackRate: number;
  pause: () => void;
}

function element(overrides: Partial<FakeMedia> = {}) {
  const media: FakeMedia = {
    readyState: 4,
    seeking: false,
    paused: true,
    currentTime: 0,
    playbackRate: 1,
    pause: vi.fn<() => void>(),
    ...overrides,
  };
  return media as unknown as HTMLMediaElement & FakeMedia;
}

const asset: VideoAsset = {
  nodeId: "source",
  sourceRevision: "revision",
  kind: "video",
  durationMs: 4_000,
  width: 1280,
  height: 720,
};
const clip = { id: "clip", assetId: "source" } as VideoClip;

function pool(presentable: boolean, settled = true) {
  const entry: PooledMedia = {
    kind: "video",
    element: {} as HTMLVideoElement,
    presentable,
  };
  return {
    prepare: vi.fn<() => PooledMedia>(() => entry),
    sync: vi.fn<() => boolean>(() => settled),
    identity: (value: VideoAsset) => value.nodeId,
    pauseAll: vi.fn<() => void>(),
    get: vi.fn<() => undefined>(() => undefined),
    retain: vi.fn<() => void>(),
    dispose: vi.fn<() => void>(),
  };
}

const plan = {
  layers: [{ clip, asset, sourceMs: 0, fade: 1, gain: 1 }],
  audio: [],
  texts: [],
  preroll: [],
  transition: undefined,
};

const materials = () => new Map([["source", { id: "source" }]]) as never;

describe("media transport", () => {
  it("parks a paused element with a single seek and reports it only once it settles", () => {
    const media = element({ currentTime: 0 });
    expect(syncParked(media, 1_000)).toBe("waiting");
    expect(media.currentTime).toBeCloseTo(1, 5);
    expect(syncParked(media, 1_000)).toBe("parked");
  });

  it("never touches an element that is already seeking", () => {
    const media = element({ seeking: true, currentTime: 0 });
    expect(syncParked(media, 2_000)).toBe("waiting");
    expect(media.currentTime).toBe(0);
    expect(syncPlaying(media, 2_000)).toBe("waiting");
    expect(media.currentTime).toBe(0);
  });

  it("absorbs playback drift with the rate and keeps the position untouched", () => {
    const media = element({ paused: false, currentTime: 1.02 });
    expect(syncPlaying(media, 1_000)).toBe("playing");
    expect(media.currentTime).toBeCloseTo(1.02, 5);
    expect(media.playbackRate).toBeCloseTo(0.968, 5);
  });

  it("repositions only when the gap is past what a rate trim can recover", () => {
    const media = element({ paused: false, currentTime: 5 });
    expect(syncPlaying(media, 1_000)).toBe("waiting");
    expect(media.currentTime).toBeCloseTo(1, 5);
    expect(media.playbackRate).toBe(1);
  });

  it("waits for metadata instead of guessing a position", () => {
    const media = element({ readyState: 0 });
    expect(syncParked(media, 500)).toBe("waiting");
    expect(syncPlaying(media, 500)).toBe("waiting");
    expect(media.currentTime).toBe(0);
  });
});

describe("frame resolution", () => {
  it("reports a frame complete only when its layer is decoded and positioned", () => {
    const ready = createFrameResolver({ pool: pool(true) as never, materials, playing: () => true });
    expect(ready.resolve(plan).complete).toBe(true);
    const decoding = createFrameResolver({
      pool: pool(false) as never,
      materials,
      playing: () => true,
    });
    const frame = decoding.resolve(plan);
    expect(frame.complete).toBe(false);
    expect(frame.layers).toEqual([]);
  });

  it("retains exactly the media the position needs, so the pool can release the rest", () => {
    const resolver = createFrameResolver({
      pool: pool(true) as never,
      materials,
      playing: () => false,
    });
    expect([...resolver.resolve(plan).retained]).toEqual(["source"]);
  });

  it("drops a layer whose material left the space instead of reporting a hole as ready", () => {
    const resolver = createFrameResolver({
      pool: pool(true) as never,
      materials: () => new Map(),
      playing: () => true,
    });
    const frame = resolver.resolve(plan);
    expect(frame.layers).toEqual([]);
    expect(frame.complete).toBe(false);
  });
});
