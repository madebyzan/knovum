import { defineStore } from "pinia";
import { computed, onScopeDispose, ref } from "vue";

import { TaskExecutionHandlers, type CancelTask, type OpenTask } from "./task-execution-handlers";
import type { AppTask, TaskStatus } from "./task-model";
import { createTaskProgress } from "./task-progress";
import { createTaskRetryController } from "./task-retry";

const SUCCESS_DISMISS_DELAY_MS = 10_000;
const TASK_HISTORY_LIMIT = 50;

export const useTaskStore = defineStore("tasks", () => {
  const tasks = ref<AppTask[]>([]);
  const { progress, indeterminate, setStatusMessage, title } = createTaskProgress(tasks);
  const completionTimers = new Map<string, ReturnType<typeof setTimeout>>();
  const handlers = new TaskExecutionHandlers();
  const externallySettled = new Set<string>();
  const running = computed(() => tasks.value.filter((task) => task.status === "running"));
  const failed = computed(() => tasks.value.filter((task) => task.status === "failed"));
  const completed = computed(() => tasks.value.filter((task) => task.status === "succeeded"));
  const retryController = createTaskRetryController(
    tasks,
    (id) => {
      if (!externallySettled.has(id)) succeed(id);
    },
    trimHistory,
  );

  function dismiss(id: string) {
    const timer = completionTimers.get(id);
    if (timer) clearTimeout(timer);
    completionTimers.delete(id);
    handlers.delete(id);
    externallySettled.delete(id);
    retryController.delete(id);
    tasks.value = tasks.value.filter((task) => task.id !== id);
  }

  function trimHistory() {
    while (tasks.value.length > TASK_HISTORY_LIMIT) {
      const removable = tasks.value.findLast((candidate) => candidate.status !== "running");
      if (!removable) return;
      dismiss(removable.id);
    }
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

  async function cancel(id: string) {
    const task = tasks.value.find((candidate) => candidate.id === id);
    const handler = handlers.cancellation(id);
    if (!task || task.status !== "running" || task.cancelling || !handler) return;
    task.cancelling = true;
    task.cancellable = false;
    try {
      await handler();
      if (!externallySettled.has(id)) finish(id, "cancelled");
    } catch (error) {
      if (externallySettled.has(id)) {
        task.cancelling = false;
        task.cancellable = true;
        task.statusMessage = String(error);
      } else finish(id, "failed", String(error));
    }
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

  function setObservedRetry(id: string, retry: () => Promise<void>) {
    retryController.setRetry(id, async () => {
      externallySettled.add(id);
      try {
        await retry();
      } catch (error) {
        externallySettled.delete(id);
        throw error;
      }
    });
  }

  function clearCompleted() {
    for (const task of completed.value) dismiss(task.id);
  }

  onScopeDispose(() => {
    for (const timer of completionTimers.values()) clearTimeout(timer);
    completionTimers.clear();
    handlers.clear();
    externallySettled.clear();
    retryController.clear();
  });

  return {
    tasks,
    running,
    failed,
    completed,
    start,
    restoreRunning,
    restoreFailed: retryController.restoreFailed,
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
    setObservedRetry,
    retry: retryController.retry,
    dismiss,
    clearCompleted,
  };
});
