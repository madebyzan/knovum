import type { Ref } from "vue";

import { getErrorMessage } from "../errors/error-format";
import type { CancelTask, OpenTask, ResolveTask, TaskExecutionHandlers } from "./task-execution-handlers";
import type { AppTask } from "./task-model";
import type { RetryTask } from "./task-retry";

export type DismissTask = () => Promise<void> | void;

export interface ObservedTask {
  task: AppTask;
  cancel?: CancelTask;
  retry?: RetryTask;
  dismiss?: DismissTask;
  open?: OpenTask;
  resolve?: ResolveTask;
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
  const dismissByTaskId = new Map<string, DismissTask>();
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
    { task: next, cancel, retry: retryTask, dismiss: dismissTask, open, resolve }: ObservedTask,
    current?: AppTask,
  ) {
    const flags = {
      producer,
      durable: true,
      cancellable: Boolean(cancel),
      retryable: canRetry(next.id, retryTask),
      dismissible: Boolean(dismissTask),
      openable: Boolean(open),
      resolvable: Boolean(resolve),
    };
    if (current) {
      const keepRunning = current.status === "running" && next.status === "running";
      Object.assign(current, next, flags, {
        cancelling: current.status === next.status ? current.cancelling : false,
        progress: keepRunning ? current.progress : undefined,
        statusMessage:
          next.status === "running" ? (keepRunning ? current.statusMessage : undefined) : next.statusMessage,
      });
    } else {
      tasks.value.unshift({ ...next, ...flags });
    }
    externallySettledTaskIds.add(next.id);
    handlers.setCancellation(next.id, cancel);
    handlers.setOpen(next.id, open);
    handlers.setResolve(next.id, resolve);
    if (retryTask) retryByTaskId.set(next.id, retryTask);
    else retryByTaskId.delete(next.id);
    if (dismissTask) dismissByTaskId.set(next.id, dismissTask);
    else dismissByTaskId.delete(next.id);
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

  async function dismiss(taskId: string) {
    const task = tasks.value.find((candidate) => candidate.id === taskId);
    const handler = dismissByTaskId.get(taskId);
    if (!task || !handler || !task.dismissible) return;
    task.dismissible = false;
    try {
      await handler();
    } catch (error) {
      task.statusMessage = getErrorMessage(error);
      task.dismissible = true;
    }
  }

  return {
    reconcile,
    retry,
    dismiss,
    handlesRetry: (taskId: string) => retryByTaskId.has(taskId),
    handlesDismiss: (taskId: string) => dismissByTaskId.has(taskId),
    delete: (taskId: string) => {
      retryByTaskId.delete(taskId);
      dismissByTaskId.delete(taskId);
      retryingTaskIds.delete(taskId);
    },
    clear: () => {
      retryByTaskId.clear();
      dismissByTaskId.clear();
      retryingTaskIds.clear();
    },
  };
}
