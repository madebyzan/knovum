import { clamp } from "es-toolkit";
import { defineStore } from "pinia";
import { computed, onScopeDispose, ref } from "vue";

import { TaskExecutionHandlers, type CancelTask } from "./task-execution-handlers";
import type { AppTask, TaskStatus } from "./task-model";
import { createTaskRetryController } from "./task-retry";

const SUCCESS_DISMISS_DELAY_MS = 10_000;
const TASK_HISTORY_LIMIT = 50;

export type { AppTask } from "./task-model";

export const useTaskStore = defineStore("tasks", () => {
  const tasks = ref<AppTask[]>([]);
  const completionTimers = new Map<string, ReturnType<typeof setTimeout>>();
  const handlers = new TaskExecutionHandlers();
  const running = computed(() => tasks.value.filter((task) => task.status === "running"));
  const failed = computed(() => tasks.value.filter((task) => task.status === "failed"));
  const completed = computed(() => tasks.value.filter((task) => task.status === "succeeded"));
  const retryController = createTaskRetryController(tasks, succeed, trimHistory);

  function dismiss(id: string) {
    const timer = completionTimers.get(id);
    if (timer) clearTimeout(timer);
    completionTimers.delete(id);
    handlers.delete(id);
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
    taskDetail?: string,
    cancelTask?: CancelTask,
    id: string = crypto.randomUUID(),
  ) {
    const task: AppTask = {
      id,
      title: taskTitle,
      status: "running",
      startedAt: Date.now(),
    };
    if (taskDetail !== undefined) task.detail = taskDetail;
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
    if (cancelTask) handlers.setCancellation(task.id, cancelTask);
    trimHistory();
    return true;
  }

  function progress(id: string, value: number, taskDetail?: string) {
    const task = tasks.value.find((candidate) => candidate.id === id);
    if (!task || task.status !== "running") return;
    task.progress = clamp(value, 0, 100);
    if (taskDetail !== undefined) task.detail = taskDetail;
  }

  function detail(id: string, value: string) {
    const task = tasks.value.find((candidate) => candidate.id === id);
    if (task?.status === "running") task.detail = value;
  }

  function title(id: string, value: string) {
    const task = tasks.value.find((candidate) => candidate.id === id);
    if (task?.status === "running") task.title = value;
  }

  function finish(id: string, status: Exclude<TaskStatus, "running">, taskDetail?: string) {
    const task = tasks.value.find((candidate) => candidate.id === id);
    if (!task || task.status !== "running") return undefined;
    handlers.deleteCancellation(id);
    task.status = status;
    if (taskDetail !== undefined) task.detail = taskDetail;
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
      finish(id, "cancelled");
    } catch (error) {
      finish(id, "failed", String(error));
    }
  }

  function setCancellation(id: string, handler?: CancelTask) {
    const task = tasks.value.find((candidate) => candidate.id === id);
    if (!task || task.status !== "running") return;
    handlers.setCancellation(id, handler);
    task.cancellable = Boolean(handler);
    task.cancelling = false;
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

  function clearCompleted() {
    for (const task of tasks.value.slice()) {
      if (task.status === "succeeded") dismiss(task.id);
    }
  }

  onScopeDispose(() => {
    for (const timer of completionTimers.values()) clearTimeout(timer);
    completionTimers.clear();
    handlers.clear();
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
    progress,
    detail,
    title,
    cancel,
    setCancellation,
    succeed,
    fail,
    setRetry: retryController.setRetry,
    retry: retryController.retry,
    dismiss,
    clearCompleted,
  };
});
