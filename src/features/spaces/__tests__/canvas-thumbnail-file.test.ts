import { beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  invoke: vi.fn<(...args: unknown[]) => Promise<unknown>>(),
}));
vi.mock("@tauri-apps/api/core", () => ({ invoke: mocks.invoke }));

import { saveCanvasThumbnailBlob } from "../assets/canvas-thumbnail-file";

describe("managed canvas thumbnail file", () => {
  beforeEach(() => {
    mocks.invoke.mockReset().mockResolvedValue("/managed/spaces/assets/canvas-thumbnail-thumbnail-id.jpg");
  });

  it("persists JPEG bytes rather than a Data URL", async () => {
    const path = await saveCanvasThumbnailBlob(
      new Blob([new Uint8Array([1, 2, 3])], {
        type: "image/jpeg",
      }),
    );
    expect(path).toBe("/managed/spaces/assets/canvas-thumbnail-thumbnail-id.jpg");
    expect(mocks.invoke).toHaveBeenCalledWith("plugin:spaces|persist_canvas_thumbnail", {
      bytes: [1, 2, 3],
    });
  });

  it("rejects an unexpected encoding", async () => {
    await expect(saveCanvasThumbnailBlob(new Blob(["png"], { type: "image/png" }))).rejects.toThrow("JPEG");
    expect(mocks.invoke).not.toHaveBeenCalled();
  });
});
