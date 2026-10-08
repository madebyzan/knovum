import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it, vi } from "vitest";

import type { SpaceImportContext } from "../assets/import-contract";

const log = vi.hoisted(() => ({ warn: vi.fn<(message: string) => void>() }));
const assets = vi.hoisted(() => ({
  cancelImport: vi.fn<(jobId: string) => Promise<void>>(),
  cancelProcessing: vi.fn<(jobId: string) => Promise<void>>(),
  copy: vi.fn<(...args: unknown[]) => Promise<unknown>>(),
  derive: vi.fn<(...args: unknown[]) => Promise<unknown>>(),
  prepare: vi.fn<(...args: unknown[]) => Promise<unknown>>(),
  remove: vi.fn<(path: string) => Promise<void>>(),
}));
vi.mock("@tauri-apps/plugin-log", () => ({ warn: log.warn }));
vi.mock("../assets/asset-operations", () => ({
  cancelSpaceAssetImport: assets.cancelImport,
  cancelSpaceAssetProcessing: assets.cancelProcessing,
  copyStagedSpaceAsset: assets.copy,
  deriveSpaceAssetContent: assets.derive,
  prepareSpaceAssetImport: assets.prepare,
  removeSpaceAsset: assets.remove,
}));

import { commitPreparedSpaceImport, importSpaceSource } from "../assets/import-session";

const labels = { title: "Import", preparing: "Preparing" };
const prepared = {
  jobId: null,
  asset: { kind: "image" as const, name: "image.png", path: "/asset.png" },
};

function context(): SpaceImportContext {
  return {
    applyNodeContentForSpace: vi.fn<SpaceImportContext["applyNodeContentForSpace"]>(),
    ensureImportTarget: vi.fn<SpaceImportContext["ensureImportTarget"]>().mockResolvedValue(undefined),
    createPendingContentForSpace: vi
      .fn<SpaceImportContext["createPendingContentForSpace"]>()
      .mockResolvedValue([
        {
          id: "node",
          type: "image",
          initialPosition: { x: 0, y: 0 },
          data: {
            kind: "image",
            title: "image.png",
            assetPath: "/asset.png",
            fileName: "image.png",
          },
        },
      ]),
    savePendingNodeContentForSpace: vi.fn<SpaceImportContext["savePendingNodeContentForSpace"]>(),
    finalizePendingContent: vi.fn<SpaceImportContext["finalizePendingContent"]>(),
    discardPendingContent: vi.fn<SpaceImportContext["discardPendingContent"]>().mockResolvedValue(undefined),
  };
}

beforeEach(() => {
  setActivePinia(createPinia());
  vi.clearAllMocks();
});

describe("Space import cleanup settlement", () => {
  it("keeps a committed import successful when source disposal fails", async () => {
    const state = context();
    const dispose = vi.fn<() => Promise<void>>().mockRejectedValue(new Error("temporary source is locked"));

    await expect(
      commitPreparedSpaceImport({
        context: state,
        spaceId: "space",
        operation: {
          source: { prepare: vi.fn<() => Promise<typeof prepared>>(), dispose },
          prepared,
          taskId: "task",
        },
        position: { x: 0, y: 0 },
      }),
    ).resolves.toHaveLength(1);

    expect(log.warn).toHaveBeenCalledWith(expect.stringContaining("temporary source is locked"));
  });

  it("preserves both preparation and disposal failures", async () => {
    const primary = new Error("prepare failed");
    const cleanup = new Error("dispose failed");
    const source = {
      prepare: vi.fn<() => Promise<typeof prepared>>().mockRejectedValue(primary),
      dispose: vi.fn<() => Promise<void>>().mockRejectedValue(cleanup),
    };

    const error = await importSpaceSource({
      context: context(),
      spaceId: "space",
      source,
      position: { x: 0, y: 0 },
      labels,
    }).catch((reason: unknown) => reason);

    expect(error).toBeInstanceOf(AggregateError);
    expect(error).toMatchObject({ cause: primary });
    expect((error as AggregateError).errors).toEqual([primary, cleanup]);
  });
});
