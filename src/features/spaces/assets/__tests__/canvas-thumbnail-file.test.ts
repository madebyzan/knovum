import { beforeEach, describe, expect, it, vi } from "vitest";

const fs = vi.hoisted(() => ({
  mkdir: vi.fn<(...args: unknown[]) => Promise<void>>(),
  writeFile: vi.fn<(...args: unknown[]) => Promise<void>>(),
}));
vi.mock("@tauri-apps/plugin-fs", () => ({ ...fs, BaseDirectory: { AppData: 1 } }));
vi.mock("@tauri-apps/api/path", () => ({
  appDataDir: () => Promise.resolve("/managed"),
  join: (...parts: string[]) => Promise.resolve(parts.join("/")),
}));

import { saveCanvasThumbnailBlob } from "../canvas-thumbnail-file";

describe("managed canvas thumbnail file", () => {
  beforeEach(() => {
    fs.mkdir.mockReset();
    fs.writeFile.mockReset();
    vi.stubGlobal("crypto", { randomUUID: () => "thumbnail-id" });
  });

  it("persists JPEG bytes rather than a Data URL", async () => {
    const path = await saveCanvasThumbnailBlob(
      new Blob([new Uint8Array([1, 2, 3])], {
        type: "image/jpeg",
      }),
    );
    expect(path).toBe("/managed/spaces/assets/canvas-thumbnail-thumbnail-id.jpg");
    expect(fs.writeFile).toHaveBeenCalledWith(
      "spaces/assets/canvas-thumbnail-thumbnail-id.jpg",
      new Uint8Array([1, 2, 3]),
      { baseDir: 1 },
    );
  });

  it("rejects an unexpected encoding", async () => {
    await expect(saveCanvasThumbnailBlob(new Blob(["png"], { type: "image/png" }))).rejects.toThrow("JPEG");
    expect(fs.writeFile).not.toHaveBeenCalled();
  });
});
