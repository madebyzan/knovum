import { beforeEach, describe, expect, it, vi } from "vitest";

const invoke = vi.hoisted(() =>
  vi.fn<(command: string, args?: { bytes?: number[] }) => Promise<string>>(),
);
vi.mock("@tauri-apps/api/core", () => ({ invoke }));

import { saveCanvasThumbnailBlob } from "../canvas-thumbnail-file";

describe("managed canvas thumbnail file", () => {
  beforeEach(() => {
    invoke.mockReset().mockResolvedValue("/managed/spaces/assets/canvas-thumbnail.jpg");
  });

  it("sends JPEG bytes to the Space-owned Rust persistence boundary", async () => {
    const path = await saveCanvasThumbnailBlob(
      new Blob([new Uint8Array([1, 2, 3])], { type: "image/jpeg" }),
    );

    expect(path).toBe("/managed/spaces/assets/canvas-thumbnail.jpg");
    expect(invoke).toHaveBeenCalledWith("plugin:spaces|persist_canvas_thumbnail", {
      bytes: [1, 2, 3],
    });
  });

  it("rejects an unexpected encoding before invoking persistence", async () => {
    await expect(saveCanvasThumbnailBlob(new Blob(["png"], { type: "image/png" }))).rejects.toThrow("JPEG");
    expect(invoke).not.toHaveBeenCalled();
  });
});
