import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { createAppError } from "../../../foundation/errors/app-error";
import {
  assets,
  imageImport,
  imageProcessing,
  settleCancellationsQuietly,
} from "./support/producer-cancellation";

const media = vi.hoisted(() => ({
  cancel: vi.fn<() => Promise<void>>(),
  request: vi.fn<() => Promise<never>>(),
}));

vi.mock(
  "../assets/asset-operations",
  async () => (await import("./support/producer-cancellation")).assetOperations,
);
vi.mock(
  "../assets/concurrency-budget",
  async () => (await import("./support/producer-cancellation")).unboundedConcurrency,
);
vi.mock(
  "../enrichment/enrichment-jobs-query",
  async () => (await import("./support/producer-cancellation")).enrichmentJobs,
);
vi.mock("../media/import/ipc", () => ({
  cancelMediaImport: media.cancel,
  requestMediaImport: media.request,
}));
vi.mock("@tauri-apps/plugin-log", () => ({ info: vi.fn<(message: string) => void>() }));

import { useTaskStore } from "../../../foundation/tasks/task-store";
import { deferred } from "../../../shared/__tests__/support/deferred";
import { commitManagedImport } from "../assets/import-transfer";
import { scheduleSpaceAssetProcessing } from "../assets/processing-scheduler";
import { acquireMediaSource } from "../media/import/acquisition";

beforeEach(() => {
  setActivePinia(createPinia());
  vi.clearAllMocks();
  settleCancellationsQuietly();
  media.cancel.mockResolvedValue(undefined);
});

describe("producer cancellation authority", () => {
  it.each(["accepted", "rejected"] as const)(
    "settles processing from the producer after cancellation is %s",
    async (cancellation) => {
      const tasks = useTaskStore();
      const failure = createAppError("asset_processing_failed", "decoder crashed");
      const producer = deferred<never>();
      if (cancellation === "rejected")
        assets.cancelProcessing.mockRejectedValueOnce(new Error("cancel rejected"));
      assets.derive.mockReturnValueOnce(producer.promise);

      scheduleSpaceAssetProcessing(imageProcessing(vi.fn<() => void>()));
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
      const producer = deferred<{ path: string }>();
      if (cancellation === "rejected")
        assets.cancelImport.mockRejectedValueOnce(new Error("cancel rejected"));
      assets.copy.mockReturnValueOnce(producer.promise);
      const pending = commitManagedImport(
        imageImport(taskId, { size: 1, modifiedAt: 1, sourceRevision: "r" }),
      );
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
      const producer = deferred<never>();
      if (cancellation === "rejected") media.cancel.mockRejectedValueOnce(new Error("cancel rejected"));
      media.request.mockReturnValueOnce(producer.promise);
      const pending = acquireMediaSource("https://example.com/video", {
        jobId: "media-job",
        nodeId: "media",
        kind: "video",
        title: "example.com/video",
      });
      await tasks.cancel(tasks.tasks[0]!.id);
      expect(tasks.tasks[0]).toMatchObject(expectedCancellationState(cancellation));
      producer.reject(failure);

      await expect(pending).rejects.toBe(failure);
      expect(tasks.tasks[0]?.status).toBe("failed");
    },
  );
});

function expectedCancellationState(cancellation: "accepted" | "rejected") {
  return cancellation === "rejected" ? { statusMessage: "cancel rejected" } : { status: "running" };
}
