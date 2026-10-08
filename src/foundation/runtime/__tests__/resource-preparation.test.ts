import { createPinia, setActivePinia } from "pinia";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import type { ResourcePreparationSnapshot } from "../../../shared/contracts/generated/ResourcePreparationSnapshot";
import { useTaskStore } from "../../tasks/task-store";
import { cancelResourcePreparation, listResourcePreparations, prepareResources } from "../local-resources";
import { restoreResourcePreparationTasks, startResourcePreparation } from "../resource-preparation";

vi.mock("@tauri-apps/plugin-log", () => ({ error: vi.fn<(message: string) => Promise<void>>() }));
vi.mock("../../i18n/i18n", () => ({ i18n: { global: { t: (key: string) => key } } }));
vi.mock("../local-resources", () => ({
  prepareResources: vi.fn<typeof prepareResources>(),
  cancelResourcePreparation: vi.fn<typeof cancelResourcePreparation>().mockResolvedValue(undefined),
  listResourcePreparations: vi.fn<typeof listResourcePreparations>().mockResolvedValue([]),
}));
vi.mock("../resource-query", () => ({
  invalidateLocalResources: vi.fn<() => Promise<void>>().mockResolvedValue(undefined),
}));

describe("resource preparation task lifetime", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.useFakeTimers();
    vi.clearAllMocks();
  });
  afterEach(() => vi.useRealTimers());

  it("waits for the actual result when cancellation races successful completion", async () => {
    let complete!: () => void;
    vi.mocked(prepareResources).mockReturnValueOnce(
      new Promise<void>((resolve) => {
        complete = resolve;
      }),
    );
    const preparation = startResourcePreparation({ kind: "transcription" }, "Transcribe");
    const tasks = useTaskStore();
    const id = tasks.running[0]!.id;

    await tasks.cancel(id);

    expect(cancelResourcePreparation).toHaveBeenCalledWith(id);
    expect(tasks.running[0]).toMatchObject({ id, cancelling: true });
    complete();
    await preparation.completion;
    expect(tasks.tasks[0]).toMatchObject({ id, status: "succeeded" });
  });

  it("recognizes backend cancellation without matching an English message", async () => {
    vi.mocked(prepareResources).mockRejectedValueOnce({
      code: "local_resource_preparation_cancelled",
      message: "准备已取消",
    });
    const preparation = startResourcePreparation({ kind: "textEmbedding" }, "Semantic search");

    await expect(preparation.completion).rejects.toMatchObject({
      cause: { code: "local_resource_preparation_cancelled" },
    });
    expect(useTaskStore().tasks[0]).toMatchObject({ status: "cancelled" });
  });

  it("keeps the combined progress when one resource becomes ready before the complete preparation", async () => {
    vi.mocked(prepareResources).mockImplementationOnce(async ({ onProgress }) => {
      onProgress({ stage: "ready", downloaded: 100, total: 400, percent: 25 });
      expect(useTaskStore().tasks[0]).toMatchObject({ status: "running", progress: 25 });
      onProgress({ stage: "downloading", downloaded: 200, total: 400, percent: 50 });
      expect(useTaskStore().tasks[0]).toMatchObject({ status: "running", progress: 50 });
    });

    await startResourcePreparation({ kind: "transcription" }, "Transcribe").completion;
    expect(useTaskStore().tasks[0]).toMatchObject({ status: "succeeded", progress: 100 });
  });

  it("preserves a real preparation failure after cancellation was rejected", async () => {
    let fail!: (error: Error) => void;
    vi.mocked(prepareResources).mockReturnValueOnce(
      new Promise<void>((_resolve, reject) => {
        fail = reject;
      }),
    );
    vi.mocked(cancelResourcePreparation).mockRejectedValueOnce(new Error("Cancellation rejected"));
    const preparation = startResourcePreparation({ kind: "transcription" }, "Transcribe");
    await expect(preparation.cancel()).rejects.toThrow("Cancellation rejected");
    const failure = new Error("Disk write failed");
    const result = preparation.completion.catch((error: unknown) => error);
    fail(failure);
    expect(await result).toBe(failure);
    expect(useTaskStore().tasks[0]?.status).toBe("failed");
  });

  it("restores a media tool request and observes its terminal state without another installation", async () => {
    const snapshot: ResourcePreparationSnapshot = {
      jobId: "resource:media-import",
      title: "Prepare media import",
      target: { kind: "mediaImportTools" },
      startedAt: 1,
      completedAt: null,
      state: "preparing",
      progress: { stage: "installing", downloaded: 20, total: null, percent: null },
      error: null,
    };
    vi.mocked(listResourcePreparations)
      .mockResolvedValueOnce([snapshot])
      .mockResolvedValueOnce([snapshot])
      .mockResolvedValueOnce([{ ...snapshot, state: "ready", completedAt: 2 }]);

    const pinia = createPinia();
    setActivePinia(pinia);
    await restoreResourcePreparationTasks(pinia);
    await vi.advanceTimersByTimeAsync(1_000);

    expect(prepareResources).not.toHaveBeenCalled();
    expect(useTaskStore().tasks).toMatchObject([
      { id: snapshot.jobId, title: snapshot.title, status: "succeeded" },
    ]);
  });

  it("keeps observing a restored task after a transient list failure", async () => {
    const snapshot: ResourcePreparationSnapshot = {
      jobId: "resource:transient",
      title: "Prepare transcription",
      target: { kind: "transcription" },
      startedAt: 1,
      completedAt: null,
      state: "preparing",
      progress: { stage: "installing", downloaded: 0, total: null, percent: null },
      error: null,
    };
    vi.mocked(listResourcePreparations)
      .mockResolvedValueOnce([snapshot])
      .mockRejectedValueOnce(new Error("temporary IPC failure"))
      .mockResolvedValueOnce([{ ...snapshot, state: "ready", completedAt: 2 }]);

    const pinia = createPinia();
    setActivePinia(pinia);
    await restoreResourcePreparationTasks(pinia);
    await vi.advanceTimersByTimeAsync(1_000);

    expect(useTaskStore().tasks[0]).toMatchObject({ id: snapshot.jobId, status: "succeeded" });
  });

  it("fails a task whose restored snapshot disappears and can observe it again on retry", async () => {
    const snapshot: ResourcePreparationSnapshot = {
      jobId: "resource:missing",
      title: "Prepare media tools",
      target: { kind: "mediaImportTools" },
      startedAt: 1,
      completedAt: null,
      state: "preparing",
      progress: { stage: "installing", downloaded: 0, total: null, percent: null },
      error: null,
    };
    vi.mocked(listResourcePreparations)
      .mockResolvedValueOnce([snapshot])
      .mockResolvedValueOnce([])
      .mockResolvedValueOnce([])
      .mockResolvedValueOnce([])
      .mockResolvedValueOnce([{ ...snapshot, state: "ready", completedAt: 2 }]);

    const pinia = createPinia();
    setActivePinia(pinia);
    await restoreResourcePreparationTasks(pinia);
    await vi.advanceTimersByTimeAsync(2_000);
    const tasks = useTaskStore();
    expect(tasks.tasks[0]).toMatchObject({ id: snapshot.jobId, status: "failed", retryable: true });

    await tasks.retry(snapshot.jobId);
    expect(tasks.tasks[0]).toMatchObject({ id: snapshot.jobId, status: "succeeded" });
    expect(prepareResources).not.toHaveBeenCalled();
  });
});
