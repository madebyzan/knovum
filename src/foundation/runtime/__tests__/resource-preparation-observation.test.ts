import { createPinia, setActivePinia } from "pinia";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import type { ResourcePreparationSnapshot } from "../../../shared/contracts/generated/ResourcePreparationSnapshot";
import { useTaskStore } from "../../tasks/task-store";
import {
  type cancelResourcePreparation,
  listResourcePreparations,
  prepareResources,
} from "../local-resources";
import { restoreResourcePreparationTasks } from "../resource-preparation";

vi.mock("@tauri-apps/plugin-log", () => ({ error: vi.fn<(message: string) => Promise<void>>() }));
vi.mock("../../i18n/instance", () => ({ i18n: { global: { t: (key: string) => key } } }));
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

  it("marks an observation outage as attention and reconnects without restarting the producer", async () => {
    const snapshot: ResourcePreparationSnapshot = {
      jobId: "resource:missing",
      title: "Prepare media tools",
      target: { kind: "mediaImportTools" },
      startedAt: 1,
      completedAt: null,
      state: "preparing",
      progress: { stage: "downloading", downloaded: 37, total: 100, percent: 37 },
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
    expect(tasks.tasks[0]).toMatchObject({
      id: snapshot.jobId,
      title: snapshot.title,
      status: "attention",
      progress: 37,
      cancellable: true,
      retryable: true,
    });

    await tasks.retry(snapshot.jobId);
    expect(tasks.tasks[0]).toMatchObject({ id: snapshot.jobId, status: "succeeded" });
    expect(prepareResources).not.toHaveBeenCalled();
  });
});
