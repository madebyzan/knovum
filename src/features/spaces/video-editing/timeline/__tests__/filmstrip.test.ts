import { describe, expect, it, vi } from "vitest";

import type { VideoLibrary } from "../../media/preview-video";
import type { VideoAsset } from "../../project";
import { createFilmstripCache } from "../filmstrip";

const decoded = vi.hoisted(() => [] as number[][]);
vi.mock("mediabunny", () => ({
  /** Key frames every two seconds, as a long-GOP camera file has them. */
  EncodedPacketSink: class {
    getKeyPacket(timestamp: number) {
      return Promise.resolve({ timestamp: Math.floor(timestamp / 2) * 2 });
    }
  },
  CanvasSink: class {
    async *canvasesAtTimestamps(timestamps: number[]) {
      decoded.push([...timestamps]);
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
  const videos = { load: () => Promise.resolve({ track: {} }) } as unknown as VideoLibrary;
  return createFilmstripCache({ videos, suspended: () => false });
}

const settle = () => new Promise((resolve) => setTimeout(resolve, 0));

describe("filmstrip tiles", () => {
  it("decodes one key frame per tile group instead of every requested position", async () => {
    decoded.length = 0;
    const filmstrips = cache();
    filmstrips.request(asset, 1_000, [0, 1, 2, 3, 4, 5]);
    await settle();
    expect(decoded).toEqual([[0, 2, 4]]);
    expect(filmstrips.tile(asset, 1_000, 1)).toEqual({ timestamp: 0 });
    expect(filmstrips.tile(asset, 1_000, 5)).toEqual({ timestamp: 4 });
  });

  it("fills two clips of the same source from one queue entry", async () => {
    decoded.length = 0;
    const filmstrips = cache();
    filmstrips.request(asset, 2_000, [10]);
    filmstrips.request(asset, 2_000, [20]);
    await settle();
    expect(decoded.flat().toSorted((left, right) => left - right)).toEqual([20, 40]);
  });
});
