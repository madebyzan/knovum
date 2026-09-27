import { createPinia, setActivePinia } from "pinia";
import { expect, it } from "vitest";

import { useTaskStore } from "../task-store";

it("does not mark an accepted Rust retry successful until its observation arrives", async () => {
  setActivePinia(createPinia());
  const tasks = useTaskStore();
  tasks.start("Job", undefined, undefined, "job");
  tasks.fail("job", "failed");
  tasks.setObservedRetry("job", async () => {});
  await tasks.retry("job");
  expect(tasks.tasks[0]?.status).toBe("running");
  tasks.succeed("job");
  expect(tasks.tasks[0]?.status).toBe("succeeded");
});
