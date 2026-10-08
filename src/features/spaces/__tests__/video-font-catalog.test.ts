import { QueryClient } from "@tanstack/vue-query";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { ZodError } from "zod";

import { videoFontCatalogQuery } from "../video-editing/font-catalog";

const mocks = vi.hoisted(() => ({ invoke: vi.fn<(...args: unknown[]) => Promise<unknown>>() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: mocks.invoke }));

let client: QueryClient;
beforeEach(() => {
  mocks.invoke.mockReset();
  client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
});
afterEach(() => client.clear());

describe("installed video font catalog", () => {
  it("validates the system font catalog and shares the cached result", async () => {
    const fonts = [{ family: "Example Sans", weights: [400, 700] }];
    mocks.invoke.mockResolvedValue(fonts);
    await expect(client.fetchQuery(videoFontCatalogQuery)).resolves.toEqual(fonts);
    await expect(client.fetchQuery(videoFontCatalogQuery)).resolves.toEqual(fonts);
    expect(mocks.invoke).toHaveBeenCalledExactlyOnceWith("plugin:spaces|list_video_fonts");
  });

  it("preserves an empty catalog as a valid empty state", async () => {
    mocks.invoke.mockResolvedValue([]);
    await expect(client.fetchQuery(videoFontCatalogQuery)).resolves.toEqual([]);
  });

  it.each([
    [{ family: "", weights: [400] }],
    [{ family: "Example Sans", weights: [] }],
    [{ family: "Example Sans", weights: [0] }],
    [{ family: "Example Sans", weights: [400], obsolete: true }],
    { fonts: [] },
  ])("rejects malformed system font catalog data: %j", async (fonts) => {
    mocks.invoke.mockResolvedValue(fonts);
    await expect(client.fetchQuery(videoFontCatalogQuery)).rejects.toBeInstanceOf(ZodError);
    expect(client.getQueryData(videoFontCatalogQuery.queryKey)).toBeUndefined();
  });

  it("preserves the IPC error without caching substitute data", async () => {
    mocks.invoke.mockRejectedValue({ code: "operation_failed", message: "Font database unavailable" });
    await expect(client.fetchQuery(videoFontCatalogQuery)).rejects.toThrow("Font database unavailable");
    expect(client.getQueryData(videoFontCatalogQuery.queryKey)).toBeUndefined();
  });
});
