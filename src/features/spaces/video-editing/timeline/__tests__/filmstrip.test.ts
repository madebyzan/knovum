import { describe, expect, it, vi } from "vitest";

import type { VideoLibrary } from "../../media/preview-video";
import type { VideoAsset } from "../../project";
import { createFilmstripCache, filmstripStepMs } from "../filmstrip";

const sink = vi.hoisted(() => ({ decoded: [] as number[][], options: [] as unknown[] }));
vi.mock("mediabunny", () => ({
  /** Key frames every two seconds, as a long-GOP camera file has them. */
  EncodedPacketSink: class {
    getKeyPacket(timestamp: number) {
      return Promise.resolve({ timestamp: Math.floor(timestamp / 2) * 2 });
    }
  },
  CanvasSink: class {
    constructor(_track: unknown, options: unknown) {
      sink.options.push(options);
    }
    async *canvasesAtTimestamps(timestamps: number[]) {
      sink.decoded.push([...timestamps]);
      for (const timestamp of timestamps) yield { canvas: { timestamp } };
    }
  },
}));

const asset: VideoAsset = {
  nodeId: "source",
  sourceRevision: "revision",
  kind: "video",
  durationMs: 60_000,
  width: 1920,
  height: 1080,
};

function cache() {
  sink.decoded.length = 0;
  sink.options.length = 0;
  const videos = { load: () => Promise.resolve({ track: {} }) } as unknown as VideoLibrary;
  return createFilmstripCache({ videos, suspended: () => false });
}

const settle = () => new Promise((resolve) => setTimeout(resolve, 0));

describe("filmstrip thumbnails", () => {
  it("never lets two neighbouring slots show the same step", () => {
    expect(filmstripStepMs(55_000)).toBe(30_000);
    expect(filmstripStepMs(1_200)).toBe(1_000);
    expect(filmstripStepMs(10)).toBe(40);
  });

  it("decodes at the lane's device-pixel height", async () => {
    const filmstrips = cache();
    filmstrips.request(asset, 1_000, [0]);
    await settle();
    expect(sink.options).toEqual([{ height: 144, poolSize: 0 }]);
  });

  it("fills slots from key frames at once, then replaces them with their exact frames", async () => {
    const filmstrips = cache();
    filmstrips.request(asset, 1_000, [0, 1, 2, 3]);
    await settle();
    expect(sink.decoded).toEqual([
      [0, 2],
      [0, 1, 2, 3],
    ]);
    expect(filmstrips.tile(asset, 1_000, 1)).toEqual({ timestamp: 1 });
    expect(filmstrips.tile(asset, 1_000, 3)).toEqual({ timestamp: 3 });
  });
});
