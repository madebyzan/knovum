import { beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  hasImage: vi.fn<() => Promise<boolean>>(),
  readImageBinary: vi.fn<() => Promise<Uint8Array>>(),
  invoke: vi.fn<() => Promise<{ path: string; sourceRevision: string }>>(),
}));

vi.mock("tauri-plugin-clipboard-api", () => ({
  hasImage: mocks.hasImage,
  readImageBinary: mocks.readImageBinary,
}));
vi.mock("@tauri-apps/api/core", () => ({ invoke: mocks.invoke }));

import { importSpaceClipboardImage } from "../clipboard/image-import";

describe("clipboard image import", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.hasImage.mockResolvedValue(true);
    mocks.readImageBinary.mockResolvedValue(new Uint8Array([137, 80, 78, 71]));
    mocks.invoke.mockResolvedValue({
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
    expect(mocks.readImageBinary).toHaveBeenCalledWith("Uint8Array");
    expect(mocks.invoke).toHaveBeenCalledWith("plugin:spaces|persist_clipboard_image", {
      bytes: [137, 80, 78, 71],
    });
  });

  it("does not invoke the persistence command when no image exists", async () => {
    mocks.hasImage.mockResolvedValue(false);
    expect(await importSpaceClipboardImage("Image")).toBeUndefined();
    expect(mocks.invoke).not.toHaveBeenCalled();
  });
});
