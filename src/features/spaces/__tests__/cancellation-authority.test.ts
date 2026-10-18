import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { createAppError } from "../../../foundation/errors/app-error";
import type { SpaceImportContext } from "../assets/import-contract";

const assets = vi.hoisted(() => ({
  cancelImport: vi.fn<() => Promise<void>>(),
  cancelProcessing: vi.fn<() => Promise<void>>(),
  copy: vi.fn<() => Promise<{ path: string }>>(),
  derive: vi.fn<() => Promise<never>>(),
  remove: vi.fn<() => Promise<void>>(),
}));
const media = vi.hoisted(() => ({
  cancel: vi.fn<() => Promise<void>>(),
  request: vi.fn<() => Promise<never>>(),
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
vi.mock("../media/import/ipc", () => ({
  cancelMediaImport: media.cancel,
  requestMediaImport: media.request,
}));
vi.mock("@tauri-apps/plugin-log", () => ({ info: vi.fn<(message: string) => void>() }));

import { useTaskStore } from "../../../foundation/tasks/task-store";
import { commitManagedImport } from "../assets/import-transfer";
import { scheduleSpaceAssetProcessing } from "../assets/processing-scheduler";
import { acquireMediaSource } from "../media/import/acquisition";

beforeEach(() => {
  setActivePinia(createPinia());
  vi.clearAllMocks();
  assets.cancelImport.mockResolvedValue(undefined);
  assets.cancelProcessing.mockResolvedValue(undefined);
  assets.remove.mockResolvedValue(undefined);
  media.cancel.mockResolvedValue(undefined);
});

describe("producer cancellation authority", () => {
  it.each(["accepted", "rejected"] as const)(
    "settles processing from the producer after cancellation is %s",
    async (cancellation) => {
      const tasks = useTaskStore();
      const failure = createAppError("asset_processing_failed", "decoder crashed");
      const producer = pendingFailure<never>();
      if (cancellation === "rejected")
        assets.cancelProcessing.mockRejectedValueOnce(new Error("cancel rejected"));
      assets.derive.mockReturnValueOnce(producer.promise);
      const apply = vi.fn<() => void>();

      scheduleSpaceAssetProcessing({
        context: { applyNodeContentForSpace: apply },
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
      const task = tasks.tasks[0]!;
      await tasks.cancel(task.id);
      expect(tasks.tasks[0]).toMatchObject(expectedCancellationState(cancellation));
      producer.reject(failure);
      await vi.waitFor(() => expect(tasks.tasks[0]?.status).toBe("failed"));

      expect(tasks.tasks[0]?.statusMessage).toBe("decoder crashed");
    },
  );

  it.each(["accepted", "rejected"] as const)(
    "does not convert an import failure after cancellation is %s",
    async (cancellation) => {
      const tasks = useTaskStore();
      const taskId = "task";
      tasks.restoreRunning({ id: taskId, title: "Import", startedAt: Date.now() });
      const failure = createAppError("asset_copy_failed", "copy failed");
      const producer = pendingFailure<{ path: string }>();
      if (cancellation === "rejected")
        assets.cancelImport.mockRejectedValueOnce(new Error("cancel rejected"));
      assets.copy.mockReturnValueOnce(producer.promise);
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
      const pending = commitManagedImport({
        context,
        spaceId: "space",
        taskId,
        prepared: {
          jobId: "job",
          asset: {
            kind: "image",
            name: "Image",
            path: "/staged",
            size: 1,
            modifiedAt: 1,
            sourceRevision: "r",
          },
        },
        position: { x: 0, y: 0 },
        reveal: true,
        source: "/image.png",
      });
      await tasks.cancel(taskId);
      expect(tasks.tasks[0]).toMatchObject(expectedCancellationState(cancellation));
      producer.reject(failure);

      await expect(pending).rejects.toBe(failure);
      expect(tasks.tasks[0]?.status).toBe("failed");
    },
  );

  it.each(["accepted", "rejected"] as const)(
    "does not convert a media failure after cancellation is %s",
    async (cancellation) => {
      const tasks = useTaskStore();
      const failure = createAppError("media_import_failed", "download failed");
      const producer = pendingFailure<never>();
      if (cancellation === "rejected") media.cancel.mockRejectedValueOnce(new Error("cancel rejected"));
      media.request.mockReturnValueOnce(producer.promise);
      const pending = acquireMediaSource("https://example.com/video", {
        resolved: vi.fn<() => Promise<{ nodeId: string; title: string }>>(),
      });
      await tasks.cancel(tasks.tasks[0]!.id);
      expect(tasks.tasks[0]).toMatchObject(expectedCancellationState(cancellation));
      producer.reject(failure);

      await expect(pending).rejects.toBe(failure);
      expect(tasks.tasks[0]?.status).toBe("failed");
    },
  );
});

function pendingFailure<T>() {
  let rejectPromise!: (reason: unknown) => void;
  const promise = new Promise<T>((_resolve, reject) => (rejectPromise = reject));
  return { promise, reject: rejectPromise };
}

function expectedCancellationState(cancellation: "accepted" | "rejected") {
  return cancellation === "rejected" ? { statusMessage: "cancel rejected" } : { status: "running" };
}
