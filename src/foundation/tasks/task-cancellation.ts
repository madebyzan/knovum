import type { Ref } from "vue";

import { getErrorMessage } from "../errors/error-format";
import type { TaskExecutionHandlers } from "./task-execution-handlers";
import type { AppTask } from "./task-model";

interface TaskCancellationOptions {
  tasks: Ref<AppTask[]>;
  handlers: TaskExecutionHandlers;
  externallySettledTaskIds: Set<string>;
  finish: (taskId: string, status: "cancelled") => AppTask | undefined;
}

export function createTaskCancellation({
  tasks,
  handlers,
  externallySettledTaskIds,
  finish,
}: TaskCancellationOptions) {
  return async (taskId: string) => {
    const task = tasks.value.find((candidate) => candidate.id === taskId);
    const handler = handlers.cancellation(taskId);
    if (!task || !["queued", "running", "attention"].includes(task.status) || task.cancelling || !handler)
      return;
    task.cancelling = true;
    task.cancellable = false;
    try {
      await handler();
      if (!externallySettledTaskIds.has(taskId)) finish(taskId, "cancelled");
    } catch (error) {
      if (!["queued", "running", "attention"].includes(task.status)) return;
      task.cancelling = false;
      task.cancellable = Boolean(handlers.cancellation(taskId));
      task.statusMessage = getErrorMessage(error);
    }
  };
}
