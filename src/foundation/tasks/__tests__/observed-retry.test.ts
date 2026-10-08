import { createPinia, setActivePinia } from "pinia";
import { expect, it, vi } from "vitest";

import { useTaskStore } from "../task-store";

it("retries an attention task by reconnecting its observer", async () => {
  setActivePinia(createPinia());
  const tasks = useTaskStore();
  const retryObservation = vi.fn<() => Promise<void>>(async () => {
    tasks.succeed("job");
  });
  tasks.restoreRunning({ id: "job", title: "Job", startedAt: 1, progress: 42 }, async () => {});
  tasks.markObservationUnavailable("job", "Unable to observe", retryObservation);

  expect(tasks.tasks[0]).toMatchObject({
    status: "attention",
    progress: 42,
    cancellable: true,
    retryable: true,
  });
  await tasks.retry("job");
  expect(retryObservation).toHaveBeenCalledOnce();
  expect(tasks.tasks[0]?.status).toBe("succeeded");
});
