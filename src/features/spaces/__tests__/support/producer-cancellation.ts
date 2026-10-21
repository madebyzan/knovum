import { vi } from "vitest";

import type { SpaceImportContext } from "../../assets/import-contract";
import type { CommitManagedImportOptions } from "../../assets/import-transfer";
import type { scheduleSpaceAssetProcessing } from "../../assets/processing-scheduler";
import type { PreparedAsset } from "../../contracts/generated/PreparedAsset";

/**
 * Producer doubles shared by the cancellation suites. Each suite routes its `vi.mock` factories
 * here, so every suite drives the same instances it asserts on.
 */
export const assets = {
  cancelImport: vi.fn<() => Promise<void>>(),
  cancelProcessing: vi.fn<() => Promise<void>>(),
  copy: vi.fn<() => Promise<{ path: string }>>(),
  derive: vi.fn<() => Promise<never>>(),
  remove: vi.fn<() => Promise<void>>(),
};

export const assetOperations = {
  cancelSpaceAssetImport: assets.cancelImport,
  cancelSpaceAssetProcessing: assets.cancelProcessing,
  copyStagedSpaceAsset: assets.copy,
  deriveSpaceAssetContent: assets.derive,
  removeSpaceAsset: assets.remove,
};

export const unboundedConcurrency = {
  runSpaceAssetProcessing: (operation: () => Promise<unknown>) => operation(),
  runSpaceAssetTransfer: (operation: () => Promise<unknown>) => operation(),
};

export const enrichmentJobs = { refreshEnrichmentJobs: vi.fn<() => Promise<void>>() };

export function settleCancellationsQuietly() {
  assets.cancelImport.mockResolvedValue(undefined);
  assets.cancelProcessing.mockResolvedValue(undefined);
  assets.remove.mockResolvedValue(undefined);
}

export function imageProcessing(
  applyNodeContentForSpace: () => void,
): Parameters<typeof scheduleSpaceAssetProcessing>[0] {
  return {
    context: { applyNodeContentForSpace },
    spaceId: "space",
    nodeId: "node",
    data: {
      kind: "image",
      title: "Image",
      assetPath: "/image.png",
      fileName: "image.png",
      processingState: "processing",
    },
  };
}

export function imageImport(taskId: string, asset: Partial<PreparedAsset> = {}): CommitManagedImportOptions {
  const context: SpaceImportContext = {
    applyNodeContentForSpace: vi.fn<() => void>(),
    ensureImportTarget: vi.fn<() => Promise<void>>(),
    createPendingContentForSpace: vi
      .fn<SpaceImportContext["createPendingContentForSpace"]>()
      .mockResolvedValue([
        {
          id: "node",
          type: "image",
          initialPosition: { x: 0, y: 0 },
          data: { kind: "image", title: "Image", assetPath: "/staged", fileName: "image.png" },
        },
      ]),
    savePendingNodeContentForSpace: vi.fn<() => Promise<void>>(),
    finalizePendingContent: vi.fn<() => void>(),
    discardPendingContent: vi.fn<() => Promise<void>>().mockResolvedValue(undefined),
  };
  return {
    context,
    spaceId: "space",
    taskId,
    prepared: { jobId: "job", asset: { kind: "image", name: "Image", path: "/staged", ...asset } },
    position: { x: 0, y: 0 },
    reveal: true,
    source: "/image.png",
  };
}
