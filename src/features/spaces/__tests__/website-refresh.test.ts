import { beforeEach, describe, expect, it, vi } from "vitest";

import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceNodePayload, SpaceWebsitePayload } from "../model/nodes/node-payload";
import { refreshSavedWebsite } from "../website/capture";

const mocks = vi.hoisted(() => ({
  fetch: vi.fn<(url: string) => Promise<unknown>>(),
  cache: vi.fn<(url: string) => Promise<string>>(),
  remove: vi.fn<(paths: string[]) => Promise<void>>(),
}));
vi.mock("../website/website-resource", () => ({
  fetchWebsiteResource: mocks.fetch,
  cacheWebsiteAsset: mocks.cache,
}));
vi.mock("../assets/asset-operations", () => ({ removeSpaceAssets: mocks.remove }));
vi.mock("@tauri-apps/plugin-log", () => ({ warn: vi.fn<(message: string) => Promise<void>>() }));

const saved: SpaceWebsitePayload = {
  kind: "website",
  title: "User title",
  url: "https://example.com/old",
  siteName: "Old site",
  description: "Old description",
  canonicalUrl: "https://example.com/old-canonical",
  faviconPath: "/assets/old.ico",
  metadataState: "ready",
};

function context() {
  const saveNodeDataForSpace = vi
    .fn<(spaceId: string, id: string, data: SpaceNodePayload) => Promise<void>>()
    .mockResolvedValue(undefined);
  return {
    activeSpaceId: "space-1",
    nodes: [
      { id: "website-1", type: "website" as const, data: { ...saved }, initialPosition: { x: 0, y: 0 } },
    ] satisfies SpaceCanvasNode[],
    addContent: vi
      .fn<(data: SpaceNodePayload, position?: { x: number; y: number }) => Promise<SpaceCanvasNode[]>>()
      .mockResolvedValue([]),
    saveNodeDataForSpace,
  };
}

describe("saved website refresh", () => {
  beforeEach(() => {
    mocks.fetch.mockReset();
    mocks.cache.mockReset();
    mocks.remove.mockReset().mockResolvedValue(undefined);
  });

  it("replaces source metadata instead of retaining unrelated old values", async () => {
    mocks.fetch.mockResolvedValue({
      kind: "document",
      metadata: { title: "New page", finalUrl: "https://example.com/new" },
    });
    const input = context();
    await refreshSavedWebsite(input, "website-1", saved.url);
    const latest = input.saveNodeDataForSpace.mock.lastCall?.[2];
    expect(latest).toMatchObject({
      title: "User title",
      url: "https://example.com/new",
      metadataState: "ready",
    });
    expect(latest).not.toHaveProperty("description");
    expect(latest).not.toHaveProperty("canonicalUrl");
    expect(latest).not.toHaveProperty("faviconPath");
  });

  it("keeps last saved knowledge when network refresh fails", async () => {
    mocks.fetch.mockRejectedValue(new Error("network unavailable"));
    const input = context();
    await refreshSavedWebsite(input, "website-1", saved.url);
    const latest = input.saveNodeDataForSpace.mock.lastCall?.[2];
    expect(latest).toMatchObject({
      description: "Old description",
      faviconPath: "/assets/old.ico",
      metadataState: "failed",
    });
  });
});
