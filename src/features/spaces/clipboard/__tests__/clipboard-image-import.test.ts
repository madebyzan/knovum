import { beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  hasImage: vi.fn<() => Promise<boolean>>(),
  readImageBinary: vi.fn<() => Promise<number[]>>(),
  persist: vi.fn<(bytes: number[]) => Promise<{ path: string; sourceRevision: string }>>(),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (command: string, args?: { bytes?: number[] }) => {
    if (command === "plugin:clipboard|has_image") return mocks.hasImage();
    if (command === "plugin:clipboard|read_image_binary") return mocks.readImageBinary();
    if (command === "plugin:spaces|persist_clipboard_image") return mocks.persist(args?.bytes ?? []);
    throw new Error(`Unexpected command: ${command}`);
  },
}));

import { importSpaceClipboardImage } from "../image-import";

describe("clipboard image import", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.hasImage.mockResolvedValue(true);
    mocks.readImageBinary.mockResolvedValue([137, 80, 78, 71]);
    mocks.persist.mockResolvedValue({
      path: "/spaces/assets/screenshot.png",
      sourceRevision: "sha256:image",
    });
  });

  it("persists a copied screenshot as a Space-owned image asset", async () => {
    expect(await importSpaceClipboardImage("Image")).toEqual({
      kind: "image",
      name: "Image",
      path: "/spaces/assets/screenshot.png",
      sourceRevision: "sha256:image",
    });
    expect(mocks.readImageBinary).toHaveBeenCalledOnce();
    expect(mocks.persist).toHaveBeenCalledWith([137, 80, 78, 71]);
  });

  it("does not invoke persistence when no image exists", async () => {
    mocks.hasImage.mockResolvedValue(false);
    expect(await importSpaceClipboardImage("Image")).toBeUndefined();
    expect(mocks.readImageBinary).not.toHaveBeenCalled();
    expect(mocks.persist).not.toHaveBeenCalled();
  });
});
