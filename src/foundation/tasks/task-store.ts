import { defineStore } from "pinia";
import { computed, onScopeDispose, ref } from "vue";

import { createTaskCancellation } from "./task-cancellation";
import { TaskExecutionHandlers, type CancelTask, type OpenTask } from "./task-execution-handlers";
import { trimTaskHistory } from "./task-history";
import type { AppTask, TaskStatus } from "./task-model";
import { createTaskObservation, type ObservedTask } from "./task-observation";
import { createTaskProgress } from "./task-progress";
import { createTaskRetryController } from "./task-retry";

const SUCCESS_DISMISS_DELAY_MS = 10_000;

export const useTaskStore = defineStore("tasks", () => {
  const tasks = ref<AppTask[]>([]);
  const { progress, indeterminate, setStatusMessage, title } = createTaskProgress(tasks);
  const completionTimers = new Map<string, ReturnType<typeof setTimeout>>();
  const handlers = new TaskExecutionHandlers();
  const externallySettled = new Set<string>();
  const running = computed(() => tasks.value.filter((task) => task.status === "running"));
  const failed = computed(() => tasks.value.filter((task) => task.status === "failed"));
  const completed = computed(() => tasks.value.filter((task) => task.status === "succeeded"));
  const retryController = createTaskRetryController({
    tasks,
    succeed: (id) => {
      if (!externallySettled.has(id)) succeed(id);
    },
    trimHistory,
    externallySettled,
  });
  const observation = createTaskObservation({ tasks, handlers, externallySettled, remove });
  const cancel = createTaskCancellation({ tasks, handlers, externallySettled, finish });

  function remove(id: string) {
    const timer = completionTimers.get(id);
    if (timer) clearTimeout(timer);
    completionTimers.delete(id);
    handlers.delete(id);
    externallySettled.delete(id);
    retryController.delete(id);
    observation.delete(id);
    tasks.value = tasks.value.filter((task) => task.id !== id);
  }

  function dismiss(id: string) {
    const task = tasks.value.find((candidate) => candidate.id === id);
    if (
      task?.durable &&
      (task.status === "queued" ||
        task.status === "running" ||
        task.status === "attention" ||
        task.status === "failed")
    )
      return;
    remove(id);
  }

  function trimHistory() {
    trimTaskHistory(tasks, remove);
  }

  function start(
    taskTitle: string,
    statusMessage?: string,
    cancelTask?: CancelTask,
    id: string = crypto.randomUUID(),
  ) {
    const task: AppTask = {
      id,
      title: taskTitle,
      status: "running",
      startedAt: Date.now(),
    };
    if (statusMessage !== undefined) task.statusMessage = statusMessage;
    if (cancelTask) {
      task.cancellable = true;
      handlers.setCancellation(task.id, cancelTask);
    }
    tasks.value.unshift(task);
    trimHistory();
    return task.id;
  }

  function restoreRunning(task: Omit<AppTask, "status">, cancelTask?: CancelTask) {
    if (tasks.value.some(({ id }) => id === task.id)) return false;
    tasks.value.unshift({ ...task, status: "running", cancellable: Boolean(cancelTask) });
    externallySettled.add(task.id);
    if (cancelTask) handlers.setCancellation(task.id, cancelTask);
    trimHistory();
    return true;
  }

  function finish(id: string, status: Exclude<TaskStatus, "running">, statusMessage?: string) {
    const task = tasks.value.find((candidate) => candidate.id === id);
    if (!task || task.status !== "running") return undefined;
    handlers.deleteCancellation(id);
    externallySettled.delete(id);
    task.status = status;
    if (statusMessage !== undefined) task.statusMessage = statusMessage;
    task.cancellable = false;
    task.cancelling = false;
    return task;
  }

  function setCancellation(id: string, handler?: CancelTask) {
    const task = tasks.value.find((candidate) => candidate.id === id);
    if (!task || task.status !== "running") return;
    handlers.setCancellation(id, handler);
    task.cancellable = Boolean(handler);
    task.cancelling = false;
  }

  function setOpen(id: string, handler?: OpenTask) {
    const task = tasks.value.find((candidate) => candidate.id === id);
    if (!task) return;
    handlers.setOpen(id, handler);
    task.openable = Boolean(handler);
  }

  function succeed(id: string) {
    const task = finish(id, "succeeded");
    if (!task) return;
    task.progress = 100;
    retryController.delete(id);
    const existing = completionTimers.get(id);
    if (existing) clearTimeout(existing);
    completionTimers.set(
      id,
      globalThis.setTimeout(() => dismiss(id), SUCCESS_DISMISS_DELAY_MS),
    );
  }

  function fail(id: string, value: string) {
    if (!finish(id, "failed", value)) return;
    const timer = completionTimers.get(id);
    if (timer) clearTimeout(timer);
    completionTimers.delete(id);
  }

  function cancelled(id: string) {
    finish(id, "cancelled");
  }

  function reconcileObserved(producer: string, entries: readonly ObservedTask[]) {
    observation.reconcile(producer, entries);
    trimHistory();
  }

  async function retry(id: string) {
    if (observation.handlesRetry(id)) await observation.retry(id);
    else await retryController.retry(id);
  }

  onScopeDispose(() => {
    for (const timer of completionTimers.values()) clearTimeout(timer);
    completionTimers.clear();
    handlers.clear();
    externallySettled.clear();
    retryController.clear();
    observation.clear();
  });

  return {
    tasks,
    running,
    failed,
    completed,
    start,
    restoreRunning,
    restoreFailed: retryController.restoreFailed,
    reconcileObserved,
    indeterminate,
    progress,
    setStatusMessage,
    title,
    cancel,
    setCancellation,
    setOpen,
    open: (id: string) => handlers.open(id),
    succeed,
    fail,
    cancelled,
    setRetry: retryController.setRetry,
    markObservationUnavailable: retryController.markObservationUnavailable,
    resumeObservation: retryController.resumeObservation,
    retry,
    dismiss,
    clearCompleted: () => completed.value.forEach(({ id }) => dismiss(id)),
  };
});
