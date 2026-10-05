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

  it("retains actionable failures while trimming terminal history", () => {
    const store = useTaskStore();
    const running = store.start("Long running");
    for (let index = 0; index < 60; index += 1) {
      const id = store.start(`Task ${index}`);
      store.fail(id, "failed");
    }

    expect(store.tasks).toHaveLength(61);
    expect(store.tasks.some((task) => task.id === running)).toBe(true);
  });

  it("reconciles one producer without dropping other work or live progress", () => {
    const store = useTaskStore();
    const other = store.start("Download");
    const task = { id: "job-1", title: "Visual analysis", status: "running" as const, startedAt: 1 };
    store.reconcileObserved("spaces-enrichment", [{ task }]);
    store.progress(task.id, 42, "Analyzing image");

    store.reconcileObserved("spaces-enrichment", [{ task: { ...task, title: "Visual analysis · Renamed" } }]);
    expect(store.tasks.find(({ id }) => id === task.id)).toMatchObject({
      title: "Visual analysis · Renamed",
      progress: 42,
      statusMessage: "Analyzing image",
    });

    store.reconcileObserved("spaces-enrichment", []);
    expect(store.tasks.map(({ id }) => id)).toEqual([other]);
  });

  it("keeps durable failures actionable and allows terminal dismissal", () => {
    const store = useTaskStore();
    const task = { id: "job-1", title: "Visual analysis", status: "failed" as const, startedAt: 1 };
    store.reconcileObserved("spaces-enrichment", [{ task }]);
    store.dismiss(task.id);
    expect(store.tasks).toHaveLength(1);

    store.reconcileObserved("spaces-enrichment", [{ task: { ...task, status: "running" } }]);
    store.reconcileObserved("spaces-enrichment", []);
    expect(store.tasks).toHaveLength(0);
  });

  it("retries failed tasks with the same task identity", async () => {
    const store = useTaskStore();
    const retry = vi.fn<() => Promise<void>>().mockResolvedValue(undefined);
    store.restoreFailed(
      { id: "persisted-job", title: "Transcribe", statusMessage: "runtime missing", startedAt: 1 },
      retry,
    );

    await store.retry("persisted-job");

    expect(retry).toHaveBeenCalledOnce();
    expect(store.tasks[0]).toMatchObject({ id: "persisted-job", status: "succeeded" });
  });

  it("keeps the actionable error when a retry fails", async () => {
    const store = useTaskStore();
    store.restoreFailed(
      { id: "persisted-job", title: "Transcribe", statusMessage: "old error", startedAt: 1 },
      () => Promise.reject(new Error("Whisper model is not installed")),
    );

    await store.retry("persisted-job");

    expect(store.tasks[0]).toMatchObject({
      id: "persisted-job",
      status: "failed",
      statusMessage: "Whisper model is not installed",
      retryable: true,
    });
  });

  it("keeps the producer running when it rejects a cancellation request", async () => {
    const store = useTaskStore();
    const id = store.start("Publish", undefined, async () => {
      throw new Error("Publication already started");
    });
    await store.cancel(id);
    expect(store.tasks[0]).toMatchObject({ status: "running", cancelling: false, cancellable: true });
    store.succeed(id);
    expect(store.tasks[0]?.status).toBe("succeeded");
  });

  it("preserves completion when a cancellation response arrives later", async () => {
    const store = useTaskStore();
    let rejectCancellation!: (error: Error) => void;
    const id = store.start(
      "Publish",
      undefined,
      () =>
        new Promise<void>((_resolve, reject) => {
          rejectCancellation = reject;
        }),
    );
    const cancelling = store.cancel(id);
    store.succeed(id);
    rejectCancellation(new Error("Already completed"));
    await cancelling;
    expect(store.tasks[0]).toMatchObject({ status: "succeeded", cancellable: false, cancelling: false });
  });
});
