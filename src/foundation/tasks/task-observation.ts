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
  externallySettled: Set<string>;
  remove: (id: string) => void;
}

export function createTaskObservation({
  tasks,
  handlers,
  externallySettled,
  remove,
}: TaskObservationOptions) {
  const retries = new Map<string, RetryTask>();
  const retrying = new Set<string>();
  const canRetry = (id: string, handler?: RetryTask) => Boolean(handler) && !retrying.has(id);

  function reconcile(producer: string, observations: readonly ObservedTask[]) {
    const previous = tasks.value.filter((task) => task.producer === producer);
    const previousById = new Map(previous.map((task) => [task.id, task]));
    const incomingIds = new Set(observations.map(({ task }) => task.id));
    for (const task of previous) if (!incomingIds.has(task.id)) remove(task.id);

    for (const entry of observations) sync(producer, entry, previousById.get(entry.task.id));
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
    externallySettled.add(next.id);
    handlers.setCancellation(next.id, cancel);
    handlers.setOpen(next.id, open);
    if (retryTask) retries.set(next.id, retryTask);
    else retries.delete(next.id);
  }

  async function retry(id: string) {
    const task = tasks.value.find((candidate) => candidate.id === id);
    const handler = retries.get(id);
    if (!task || task.status !== "failed" || !handler || !task.retryable) return;
    retrying.add(id);
    task.retryable = false;
    try {
      await handler();
    } catch (error) {
      task.statusMessage = getErrorMessage(error);
    } finally {
      retrying.delete(id);
      if (task.status === "failed") task.retryable = true;
    }
  }

  return {
    reconcile,
    retry,
    handlesRetry: (id: string) => retries.has(id),
    delete: (id: string) => {
      retries.delete(id);
      retrying.delete(id);
    },
    clear: () => {
      retries.clear();
      retrying.clear();
    },
  };
}
