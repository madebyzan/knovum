import type { Ref } from "vue";

import { getErrorMessage } from "../errors/error-format";
import type { TaskExecutionHandlers } from "./task-execution-handlers";
import type { AppTask } from "./task-model";

interface TaskCancellationOptions {
  tasks: Ref<AppTask[]>;
  handlers: TaskExecutionHandlers;
  externallySettled: Set<string>;
  finish: (id: string, status: "cancelled" | "failed", message?: string) => AppTask | undefined;
}

export function createTaskCancellation({
  tasks,
  handlers,
  externallySettled,
  finish,
}: TaskCancellationOptions) {
  return async (id: string) => {
    const task = tasks.value.find((candidate) => candidate.id === id);
    const handler = handlers.cancellation(id);
    if (!task || !["queued", "running", "attention"].includes(task.status) || task.cancelling || !handler)
      return;
    task.cancelling = true;
    task.cancellable = false;
    try {
      await handler();
      if (!externallySettled.has(id)) finish(id, "cancelled");
    } catch (error) {
      if (externallySettled.has(id)) {
        task.cancelling = false;
        task.cancellable = true;
        task.statusMessage = getErrorMessage(error);
      } else finish(id, "failed", getErrorMessage(error));
    }
  };
}
