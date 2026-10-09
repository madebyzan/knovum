import type { Ref } from "vue";

import { getErrorMessage } from "../errors/error-format";
import type { CancelTask, OpenTask, TaskExecutionHandlers } from "./task-execution-handlers";
import type { AppTask } from "./task-model";
import type { RetryTask } from "./task-retry";

export interface ObservedTask {
  task: AppTask;
  cancel?: CancelTask;
  retry?: RetryTask;
  open?: OpenTask;
}

interface TaskObservationOptions {
  tasks: Ref<AppTask[]>;
  handlers: TaskExecutionHandlers;
  externallySettledTaskIds: Set<string>;
  remove: (taskId: string) => void;
}

export function createTaskObservation({
  tasks,
  handlers,
  externallySettledTaskIds,
  remove,
}: TaskObservationOptions) {
  const retryByTaskId = new Map<string, RetryTask>();
  const retryingTaskIds = new Set<string>();
  const canRetry = (taskId: string, handler?: RetryTask) => Boolean(handler) && !retryingTaskIds.has(taskId);

  function reconcile(producer: string, observations: readonly ObservedTask[]) {
    const previous = tasks.value.filter((task) => task.producer === producer);
    const previousTaskById = new Map(previous.map((task) => [task.id, task]));
    const incomingIds = new Set(observations.map(({ task }) => task.id));
    for (const task of previous) if (!incomingIds.has(task.id)) remove(task.id);

    for (const entry of observations) sync(producer, entry, previousTaskById.get(entry.task.id));
  }

  function sync(
    producer: string,
    { task: next, cancel, retry: retryTask, open }: ObservedTask,
    current?: AppTask,
  ) {
    if (current) {
      const progress =
        current.status === "running" && next.status === "running" ? current.progress : undefined;
      const stage =
        current.status === "running" && next.status === "running" ? current.statusMessage : undefined;
      Object.assign(current, next, {
        producer,
        durable: true,
        cancellable: Boolean(cancel),
        retryable: canRetry(next.id, retryTask),
        openable: Boolean(open),
        cancelling: current.status === next.status ? current.cancelling : false,
        progress,
        statusMessage: next.status === "running" ? stage : next.statusMessage,
      });
    } else {
      tasks.value.unshift({
        ...next,
        producer,
        durable: true,
        cancellable: Boolean(cancel),
        retryable: canRetry(next.id, retryTask),
        openable: Boolean(open),
      });
    }
    externallySettledTaskIds.add(next.id);
    handlers.setCancellation(next.id, cancel);
    handlers.setOpen(next.id, open);
    if (retryTask) retryByTaskId.set(next.id, retryTask);
    else retryByTaskId.delete(next.id);
  }

  async function retry(taskId: string) {
    const task = tasks.value.find((candidate) => candidate.id === taskId);
    const handler = retryByTaskId.get(taskId);
    if (!task || task.status !== "failed" || !handler || !task.retryable) return;
    retryingTaskIds.add(taskId);
    task.retryable = false;
    try {
      await handler();
    } catch (error) {
      task.statusMessage = getErrorMessage(error);
    } finally {
      retryingTaskIds.delete(taskId);
      if (task.status === "failed") task.retryable = true;
    }
  }

  return {
    reconcile,
    retry,
    handlesRetry: (taskId: string) => retryByTaskId.has(taskId),
    delete: (taskId: string) => {
      retryByTaskId.delete(taskId);
      retryingTaskIds.delete(taskId);
    },
    clear: () => {
      retryByTaskId.clear();
      retryingTaskIds.clear();
    },
  };
}
