import { beforeEach, describe, expect, it, vi } from "vitest";

const bridge = vi.hoisted(() => ({ invoke: vi.fn<(...args: unknown[]) => Promise<void>>() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: bridge.invoke, Channel: vi.fn<() => void>() }));
import { removeSpaceAsset, removeSpaceAssets } from "../assets/asset-operations";

beforeEach(() => {
  bridge.invoke.mockReset();
  bridge.invoke.mockResolvedValue(undefined);
});

describe("batch asset cleanup", () => {
  it("deduplicates candidates into one native request", async () => {
    await removeSpaceAssets(["/assets/a", "/assets/b", "/assets/a"]);
    expect(bridge.invoke).toHaveBeenCalledOnce();
    expect(bridge.invoke).toHaveBeenCalledWith("plugin:spaces|remove_assets", {
      sourcePaths: ["/assets/a", "/assets/b"],
    });
  });
  it("does not query references for an empty batch", async () => {
    await removeSpaceAssets([]);
    expect(bridge.invoke).not.toHaveBeenCalled();
  });
  it("uses the same batch boundary for one file and preserves failures", async () => {
    const failure = new Error("asset is referenced");
    bridge.invoke.mockRejectedValue(failure);
    await expect(removeSpaceAsset("/assets/a")).rejects.toBe(failure);
    expect(bridge.invoke).toHaveBeenCalledWith("plugin:spaces|remove_assets", { sourcePaths: ["/assets/a"] });
  });
});
