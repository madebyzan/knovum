import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { createAppError } from "../../../foundation/errors/app-error";
import type { SpaceImportContext } from "../assets/import-contract";

const assets = vi.hoisted(() => ({
  cancelImport: vi.fn<() => Promise<void>>(),
  cancelProcessing: vi.fn<() => Promise<void>>(),
  copy: vi.fn<() => Promise<never>>(),
  derive: vi.fn<() => Promise<never>>(),
  remove: vi.fn<() => Promise<void>>(),
}));
vi.mock("../assets/asset-operations", () => ({
  cancelSpaceAssetImport: assets.cancelImport,
  cancelSpaceAssetProcessing: assets.cancelProcessing,
  copyStagedSpaceAsset: assets.copy,
  deriveSpaceAssetContent: assets.derive,
  removeSpaceAsset: assets.remove,
}));
vi.mock("../assets/concurrency-budget", () => ({
  runSpaceAssetProcessing: (operation: () => Promise<unknown>) => operation(),
  runSpaceAssetTransfer: (operation: () => Promise<unknown>) => operation(),
}));
vi.mock("../enrichment/enrichment-jobs-query", () => ({
  refreshEnrichmentJobs: vi.fn<() => Promise<void>>(),
}));

import { useTaskStore } from "../../../foundation/tasks/task-store";
import { commitManagedImport } from "../assets/import-transfer";
import { scheduleSpaceAssetProcessing } from "../assets/processing-scheduler";

beforeEach(() => {
  setActivePinia(createPinia());
  vi.clearAllMocks();
  assets.cancelImport.mockResolvedValue(undefined);
  assets.cancelProcessing.mockResolvedValue(undefined);
  assets.remove.mockResolvedValue(undefined);
});

describe("producer cancellation classification", () => {
  it("classifies processing cancellation only from the producer code", async () => {
    const tasks = useTaskStore();
    assets.derive.mockRejectedValue(
      createAppError("space_asset_processing_cancelled", "processing cancelled"),
    );
    scheduleSpaceAssetProcessing({
      context: { applyNodeContentForSpace: vi.fn<() => void>() },
      spaceId: "space",
      nodeId: "node",
      data: {
        kind: "image",
        title: "Image",
        assetPath: "/image.png",
        fileName: "image.png",
        processingState: "processing",
      },
    });

    await vi.waitFor(() => expect(tasks.tasks[0]?.status).toBe("cancelled"));
  });

  it("classifies import cancellation only from the producer code", async () => {
    const tasks = useTaskStore();
    const taskId = "task";
    tasks.restoreRunning({ id: taskId, title: "Import", startedAt: Date.now() });
    assets.copy.mockRejectedValue(createAppError("space_asset_import_cancelled", "import cancelled"));
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

    await expect(
      commitManagedImport({
        context,
        spaceId: "space",
        taskId,
        prepared: { jobId: "job", asset: { kind: "image", name: "Image", path: "/staged" } },
        position: { x: 0, y: 0 },
        reveal: true,
        source: "/image.png",
      }),
    ).resolves.toEqual([]);
    expect(tasks.tasks[0]?.status).toBe("cancelled");
  });
});
