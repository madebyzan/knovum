import { createPinia, setActivePinia } from "pinia";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { useTaskStore } from "../task-store";

describe("task store", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.useFakeTimers();
  });
  afterEach(() => vi.useRealTimers());

  it("auto-dismisses successful tasks while retaining failed tasks", () => {
    const store = useTaskStore();
    const success = store.start("Upload");
    const failed = store.start("Index");

    store.succeed(success);
    store.fail(failed, "offline");
    vi.advanceTimersByTime(10_000);

    expect(store.tasks.map((task) => task.id)).toEqual([failed]);
    expect(store.failed).toHaveLength(1);
  });

  it("clears successful history without dismissing failures", () => {
    const store = useTaskStore();
    const success = store.start("Done");
    const failed = store.start("Failed");
    store.succeed(success);
    store.fail(failed, "reason");

    store.clearCompleted();

    expect(store.tasks.map((task) => task.id)).toEqual([failed]);
  });

  it("caps historical tasks while never evicting running work", () => {
    const store = useTaskStore();
    const running = store.start("Long running");
    for (let index = 0; index < 60; index += 1) {
      const id = store.start(`Task ${index}`);
      store.fail(id, "failed");
    }

    expect(store.tasks).toHaveLength(50);
    expect(store.tasks.some((task) => task.id === running)).toBe(true);
  });

  it("retries failed tasks with the same task identity", async () => {
    const store = useTaskStore();
    const retry = vi.fn<() => Promise<void>>().mockResolvedValue(undefined);
    store.restoreFailed(
      { id: "persisted-job", title: "Transcribe", detail: "runtime missing", startedAt: 1 },
      retry,
    );

    await store.retry("persisted-job");

    expect(retry).toHaveBeenCalledOnce();
    expect(store.tasks[0]).toMatchObject({ id: "persisted-job", status: "succeeded" });
  });

  it("keeps the actionable error when a retry fails", async () => {
    const store = useTaskStore();
    store.restoreFailed({ id: "persisted-job", title: "Transcribe", detail: "old error", startedAt: 1 }, () =>
      Promise.reject(new Error("Whisper model is not installed")),
    );

    await store.retry("persisted-job");

    expect(store.tasks[0]).toMatchObject({
      id: "persisted-job",
      status: "failed",
      detail: "Error: Whisper model is not installed",
      retryable: true,
    });
  });
});
