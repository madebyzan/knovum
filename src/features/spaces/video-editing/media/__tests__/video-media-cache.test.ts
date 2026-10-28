import { describe, expect, it } from "vitest";

import type { VideoAsset } from "../../project";
import { assetIdentity, createAssetCache } from "../asset-cache";

const asset: VideoAsset = {
  nodeId: "source",
  sourceRevision: "revision",
  kind: "video",
  durationMs: 4_000,
  width: 1280,
  height: 720,
};

describe("asset cache", () => {
  it("opens media once per revision and shares concurrent requests", async () => {
    let opens = 0;
    const cache = createAssetCache(() => {
      opens += 1;
      return Promise.resolve(`opened-${opens}`);
    });
    const [first, second] = await Promise.all([cache.load(asset), cache.load(asset)]);
    expect([first, second]).toEqual(["opened-1", "opened-1"]);
    expect(await cache.load(asset)).toBe("opened-1");
    expect(opens).toBe(1);
    expect(cache.ready(asset)).toBe("opened-1");
    expect(cache.loaded(asset)).toBe(true);
    expect(await cache.load({ ...asset, sourceRevision: "next" })).toBe("opened-2");
  });

  it("does not remember a failed open, and forgets on request", async () => {
    let attempts = 0;
    const cache = createAssetCache(() => {
      attempts += 1;
      return attempts === 1 ? Promise.reject(new Error("unavailable")) : Promise.resolve("opened");
    });
    await expect(cache.load(asset)).rejects.toThrow("unavailable");
    expect(cache.loaded(asset)).toBe(false);
    expect(await cache.load(asset)).toBe("opened");
    expect(cache.forget(asset)).toBe("opened");
    expect(cache.loaded(asset)).toBe(false);
  });

  it("keys media by node and revision together", () => {
    expect(assetIdentity(asset)).not.toBe(assetIdentity({ ...asset, sourceRevision: "other" }));
    expect(assetIdentity(asset)).toBe(assetIdentity({ ...asset, durationMs: 1 }));
  });
});
