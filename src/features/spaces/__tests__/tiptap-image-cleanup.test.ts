import { describe, expect, it, vi } from "vitest";

const bridge = vi.hoisted(() => ({
  invoke: vi.fn<(command: string, args?: unknown) => Promise<unknown>>(),
}));
const assets = vi.hoisted(() => ({
  remove: vi.fn<(paths: readonly string[]) => Promise<void>>(),
}));
vi.mock("@tauri-apps/api/core", () => ({
  convertFileSrc: (path: string) => `asset://${path}`,
  invoke: bridge.invoke,
}));
vi.mock("../assets/asset-operations", () => ({ removeSpaceAssets: assets.remove }));

import { insertSpaceTiptapImagePaths, type SpaceTiptapImageEditor } from "../editor/tiptap-images";

interface ChainStub {
  focus(): ChainStub;
  insertContent(): ChainStub;
  insertContentAt(): ChainStub;
  run(): boolean;
}

describe("Tiptap image rollback", () => {
  it("preserves insertion and staged asset cleanup failures", async () => {
    const primaryMessage = "The editor rejected the image insertion";
    const cleanup = new Error("asset locked");
    bridge.invoke.mockResolvedValue("/assets/image.png");
    assets.remove.mockRejectedValue(cleanup);
    const chain: ChainStub = {
      focus: () => chain,
      insertContent: () => chain,
      insertContentAt: () => chain,
      run: () => false,
    };

    const error = await insertSpaceTiptapImagePaths(
      { isDestroyed: false, chain: () => chain } as unknown as SpaceTiptapImageEditor,
      ["/source/image.png"],
    ).catch((reason: unknown) => reason);

    expect(error).toBeInstanceOf(AggregateError);
    expect(error).toMatchObject({ cause: expect.objectContaining({ message: primaryMessage }) });
    expect((error as AggregateError).errors).toEqual([
      expect.objectContaining({ message: primaryMessage }),
      cleanup,
    ]);
  });
});
