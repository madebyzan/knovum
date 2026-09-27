import { QueryClient } from "@tanstack/vue-query";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { ZodError } from "zod";

import { videoFontCatalogQuery } from "../video-editing/font-catalog";

const native = vi.hoisted(() => ({ invoke: vi.fn<(...args: unknown[]) => Promise<unknown>>() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: native.invoke }));

let client: QueryClient;
beforeEach(() => {
  native.invoke.mockReset();
  client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
});
afterEach(() => client.clear());

describe("installed video font catalog", () => {
  it("validates the native catalog and shares the cached result", async () => {
    const fonts = [{ family: "Example Sans", weights: [400, 700] }];
    native.invoke.mockResolvedValue(fonts);
    await expect(client.fetchQuery(videoFontCatalogQuery)).resolves.toEqual(fonts);
    await expect(client.fetchQuery(videoFontCatalogQuery)).resolves.toEqual(fonts);
    expect(native.invoke).toHaveBeenCalledExactlyOnceWith("plugin:spaces|list_video_fonts");
  });

  it("preserves an empty catalog as a valid empty state", async () => {
    native.invoke.mockResolvedValue([]);
    await expect(client.fetchQuery(videoFontCatalogQuery)).resolves.toEqual([]);
  });

  it.each([
    [{ family: "", weights: [400] }],
    [{ family: "Example Sans", weights: [] }],
    [{ family: "Example Sans", weights: [0] }],
    [{ family: "Example Sans", weights: [400], obsolete: true }],
    { fonts: [] },
  ])("rejects malformed native catalog data: %j", async (fonts) => {
    native.invoke.mockResolvedValue(fonts);
    await expect(client.fetchQuery(videoFontCatalogQuery)).rejects.toBeInstanceOf(ZodError);
    expect(client.getQueryData(videoFontCatalogQuery.queryKey)).toBeUndefined();
  });

  it("preserves the native error without caching substitute data", async () => {
    native.invoke.mockRejectedValue({ code: "operation_failed", message: "Font database unavailable" });
    await expect(client.fetchQuery(videoFontCatalogQuery)).rejects.toThrow("Font database unavailable");
    expect(client.getQueryData(videoFontCatalogQuery.queryKey)).toBeUndefined();
  });
});
