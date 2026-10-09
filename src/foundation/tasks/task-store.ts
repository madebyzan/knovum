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
  const completionTimerByTaskId = new Map<string, ReturnType<typeof setTimeout>>();
  const handlers = new TaskExecutionHandlers();
  const externallySettledTaskIds = new Set<string>();
  const running = computed(() => tasks.value.filter((task) => task.status === "running"));
  const failed = computed(() => tasks.value.filter((task) => task.status === "failed"));
  const completed = computed(() => tasks.value.filter((task) => task.status === "succeeded"));
  const retryController = createTaskRetryController({
    tasks,
    succeed: (taskId) => {
      if (!externallySettledTaskIds.has(taskId)) succeed(taskId);
    },
    trimHistory,
    externallySettledTaskIds,
  });
  const observation = createTaskObservation({ tasks, handlers, externallySettledTaskIds, remove });
  const cancel = createTaskCancellation({ tasks, handlers, externallySettledTaskIds, finish });

  function remove(taskId: string) {
    const timer = completionTimerByTaskId.get(taskId);
    if (timer) clearTimeout(timer);
    completionTimerByTaskId.delete(taskId);
    handlers.delete(taskId);
    externallySettledTaskIds.delete(taskId);
    retryController.delete(taskId);
    observation.delete(taskId);
    tasks.value = tasks.value.filter((task) => task.id !== taskId);
  }

  function dismiss(taskId: string) {
    const task = tasks.value.find((candidate) => candidate.id === taskId);
    if (
      task?.durable &&
      (task.status === "queued" ||
        task.status === "running" ||
        task.status === "attention" ||
        task.status === "failed")
    )
      return;
    remove(taskId);
  }

  function trimHistory() {
    trimTaskHistory(tasks, remove);
  }

  function start(
    taskTitle: string,
    statusMessage?: string,
    cancelTask?: CancelTask,
    taskId: string = crypto.randomUUID(),
  ) {
    const task: AppTask = {
      id: taskId,
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
    externallySettledTaskIds.add(task.id);
    if (cancelTask) handlers.setCancellation(task.id, cancelTask);
    trimHistory();
    return true;
  }

  function finish(taskId: string, status: Exclude<TaskStatus, "running">, statusMessage?: string) {
    const task = tasks.value.find((candidate) => candidate.id === taskId);
    if (!task || task.status !== "running") return undefined;
    handlers.deleteCancellation(taskId);
    externallySettledTaskIds.delete(taskId);
    task.status = status;
    if (statusMessage !== undefined) task.statusMessage = statusMessage;
    task.cancellable = false;
    task.cancelling = false;
    return task;
  }

  function setCancellation(taskId: string, handler?: CancelTask) {
    const task = tasks.value.find((candidate) => candidate.id === taskId);
    if (!task || task.status !== "running") return;
    handlers.setCancellation(taskId, handler);
    task.cancellable = Boolean(handler);
    task.cancelling = false;
  }

  function setOpen(taskId: string, handler?: OpenTask) {
    const task = tasks.value.find((candidate) => candidate.id === taskId);
    if (!task) return;
    handlers.setOpen(taskId, handler);
    task.openable = Boolean(handler);
  }

  function succeed(taskId: string) {
    const task = finish(taskId, "succeeded");
    if (!task) return;
    task.progress = 100;
    retryController.delete(taskId);
    const existing = completionTimerByTaskId.get(taskId);
    if (existing) clearTimeout(existing);
    completionTimerByTaskId.set(
      taskId,
      globalThis.setTimeout(() => dismiss(taskId), SUCCESS_DISMISS_DELAY_MS),
    );
  }

  function fail(taskId: string, value: string) {
    if (!finish(taskId, "failed", value)) return;
    const timer = completionTimerByTaskId.get(taskId);
    if (timer) clearTimeout(timer);
    completionTimerByTaskId.delete(taskId);
  }

  function cancelled(taskId: string) {
    finish(taskId, "cancelled");
  }

  function reconcileObserved(producer: string, entries: readonly ObservedTask[]) {
    observation.reconcile(producer, entries);
    trimHistory();
  }

  async function retry(taskId: string) {
    if (observation.handlesRetry(taskId)) await observation.retry(taskId);
    else await retryController.retry(taskId);
  }

  onScopeDispose(() => {
    for (const timer of completionTimerByTaskId.values()) clearTimeout(timer);
    completionTimerByTaskId.clear();
    handlers.clear();
    externallySettledTaskIds.clear();
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
    open: (taskId: string) => handlers.open(taskId),
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
