import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { createAppError } from "../../../foundation/errors/app-error";
import {
  assets,
  imageImport,
  imageProcessing,
  settleCancellationsQuietly,
} from "./support/producer-cancellation";

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

import { useTaskStore } from "../../../foundation/tasks/task-store";
import { commitManagedImport } from "../assets/import-transfer";
import { scheduleSpaceAssetProcessing } from "../assets/processing-scheduler";

beforeEach(() => {
  setActivePinia(createPinia());
  vi.clearAllMocks();
  settleCancellationsQuietly();
});

describe("producer cancellation classification", () => {
  it("classifies processing cancellation only from the producer code", async () => {
    const tasks = useTaskStore();
    assets.derive.mockRejectedValue(
      createAppError("space_asset_processing_cancelled", "processing cancelled"),
    );
    scheduleSpaceAssetProcessing(imageProcessing(vi.fn<() => void>()));

    await vi.waitFor(() => expect(tasks.tasks[0]?.status).toBe("cancelled"));
  });

  it("classifies import cancellation only from the producer code", async () => {
    const tasks = useTaskStore();
    const taskId = "task";
    tasks.restoreRunning({ id: taskId, title: "Import", startedAt: Date.now() });
    assets.copy.mockRejectedValue(createAppError("space_asset_import_cancelled", "import cancelled"));

    await expect(commitManagedImport(imageImport(taskId))).resolves.toEqual([]);
    expect(tasks.tasks[0]?.status).toBe("cancelled");
  });
});
