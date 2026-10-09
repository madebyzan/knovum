import { createPinia, setActivePinia } from "pinia";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import type { ResourcePreparationSnapshot } from "../../../shared/contracts/generated/ResourcePreparationSnapshot";
import { useTaskStore } from "../../tasks/task-store";
import { cancelResourcePreparation, listResourcePreparations, prepareResources } from "../local-resources";
import { restoreResourcePreparationTasks } from "../resource-preparation";

vi.mock("@tauri-apps/plugin-log", () => ({ error: vi.fn<(message: string) => Promise<void>>() }));
vi.mock("../../i18n/i18n", () => ({ i18n: { global: { t: (key: string) => key } } }));
vi.mock("../local-resources", () => ({
  prepareResources: vi.fn<typeof prepareResources>(),
  cancelResourcePreparation: vi.fn<typeof cancelResourcePreparation>().mockResolvedValue(undefined),
  listResourcePreparations: vi.fn<typeof listResourcePreparations>().mockResolvedValue([]),
}));
vi.mock("../use-resource-queries", () => ({
  invalidateLocalResources: vi.fn<() => Promise<void>>().mockResolvedValue(undefined),
}));

describe("resource preparation task lifetime", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.useFakeTimers();
    vi.clearAllMocks();
  });
  afterEach(() => vi.useRealTimers());

  it("resumes observation after cancelling an attention task and applies its authoritative state", async () => {
    const snapshot: ResourcePreparationSnapshot = {
      jobId: "resource:cancel-observation",
      title: "Prepare media tools",
      target: { kind: "mediaImportTools" },
      startedAt: 1,
      completedAt: null,
      state: "preparing",
      progress: { stage: "installing", downloaded: 20, total: null, percent: null },
      error: null,
    };
    vi.mocked(listResourcePreparations)
      .mockResolvedValueOnce([snapshot])
      .mockResolvedValueOnce([])
      .mockResolvedValueOnce([])
      .mockResolvedValueOnce([])
      .mockResolvedValueOnce([{ ...snapshot, state: "cancelled", completedAt: 3 }]);

    const pinia = createPinia();
    setActivePinia(pinia);
    await restoreResourcePreparationTasks(pinia);
    await vi.advanceTimersByTimeAsync(2_000);
    const tasks = useTaskStore();
    expect(tasks.tasks[0]).toMatchObject({ status: "attention", cancellable: true });

    await tasks.cancel(snapshot.jobId);
    await vi.advanceTimersByTimeAsync(0);

    expect(cancelResourcePreparation).toHaveBeenCalledWith(snapshot.jobId);
    expect(tasks.tasks[0]).toMatchObject({ status: "cancelled", cancelling: false });
    expect(prepareResources).not.toHaveBeenCalled();
  });

  it("retries a failed producer through the same cancellation and settlement lifecycle", async () => {
    const snapshot: ResourcePreparationSnapshot = {
      jobId: "resource:failed",
      title: "Prepare transcription",
      target: { kind: "transcription" },
      startedAt: 1,
      completedAt: 2,
      state: "failed",
      progress: { stage: "downloading", downloaded: 12, total: 100, percent: 12 },
      error: { code: "download_failed", message: "Network unavailable" },
    };
    let fail!: (error: Error) => void;
    vi.mocked(listResourcePreparations).mockResolvedValueOnce([snapshot]);
    vi.mocked(prepareResources).mockReturnValueOnce(
      new Promise<void>((_resolve, reject) => {
        fail = reject;
      }),
    );
    vi.mocked(cancelResourcePreparation).mockRejectedValueOnce(new Error("Cancellation rejected"));

    const pinia = createPinia();
    setActivePinia(pinia);
    await restoreResourcePreparationTasks(pinia);
    const tasks = useTaskStore();
    const retry = tasks.retry(snapshot.jobId);
    await Promise.resolve();
    await tasks.cancel(snapshot.jobId);
    expect(cancelResourcePreparation).toHaveBeenCalledWith(snapshot.jobId);
    expect(tasks.tasks[0]).toMatchObject({ status: "running", cancellable: true, cancelling: false });

    fail(new Error("Checksum mismatch"));
    await retry;
    expect(tasks.tasks[0]).toMatchObject({
      status: "failed",
      statusMessage: "Checksum mismatch",
      retryable: true,
    });

    let failAfterAcceptedCancel!: (error: Error) => void;
    vi.mocked(prepareResources).mockReturnValueOnce(
      new Promise<void>((_resolve, reject) => {
        failAfterAcceptedCancel = reject;
      }),
    );
    const secondRetry = tasks.retry(snapshot.jobId);
    await Promise.resolve();
    await tasks.cancel(snapshot.jobId);
    expect(tasks.tasks[0]).toMatchObject({ status: "running", cancelling: true });
    failAfterAcceptedCancel(new Error("Producer failed after accepted cancellation"));
    await secondRetry;
    expect(tasks.tasks[0]).toMatchObject({
      status: "failed",
      statusMessage: "Producer failed after accepted cancellation",
      retryable: true,
    });

    let cancelProducer!: (error: { code: string; message: string }) => void;
    vi.mocked(prepareResources).mockReturnValueOnce(
      new Promise<void>((_resolve, reject) => {
        cancelProducer = reject;
      }),
    );
    const thirdRetry = tasks.retry(snapshot.jobId);
    await Promise.resolve();
    await tasks.cancel(snapshot.jobId);
    expect(tasks.tasks[0]).toMatchObject({ status: "running", cancelling: true });
    cancelProducer({ code: "local_resource_preparation_cancelled", message: "cancelled" });
    await thirdRetry;
    expect(tasks.tasks[0]).toMatchObject({ status: "cancelled", cancelling: false });
  });
});
