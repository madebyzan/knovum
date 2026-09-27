import { createPinia, setActivePinia } from "pinia";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { useTaskStore } from "../../tasks/task-store";
import {
  cancelResourcePreparation,
  listResourcePreparations,
  prepareResources,
  type ResourcePreparationSnapshot,
} from "../local-resources";
import {
  ResourcePreparationCancelledError,
  restoreResourcePreparationTasks,
  startResourcePreparation,
} from "../resource-preparation";

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

    await expect(preparation.completion).rejects.toBeInstanceOf(ResourcePreparationCancelledError);
    expect(useTaskStore().tasks[0]).toMatchObject({ status: "cancelled" });
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
});
