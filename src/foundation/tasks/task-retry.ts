import type { Ref } from "vue";

import type { AppTask } from "./task-model";

export type RetryTask = () => Promise<void>;

export function createTaskRetryController(
  tasks: Ref<AppTask[]>,
  succeed: (id: string) => void,
  trimHistory: () => void,
) {
  const retries = new Map<string, RetryTask>();

  function restoreFailed(task: Omit<AppTask, "status">, retryTask: RetryTask) {
    if (tasks.value.some(({ id }) => id === task.id)) return false;
    tasks.value.unshift({ ...task, status: "failed", retryable: true });
    retries.set(task.id, retryTask);
    trimHistory();
    return true;
  }

  function setRetry(id: string, retryTask: RetryTask) {
    const task = tasks.value.find((candidate) => candidate.id === id);
    if (!task || task.status !== "failed") return;
    retries.set(id, retryTask);
    task.retryable = true;
  }

  async function retry(id: string) {
    const task = tasks.value.find((candidate) => candidate.id === id);
    const retryTask = retries.get(id);
    if (!task || task.status !== "failed" || !retryTask) return;
    task.status = "running";
    task.retryable = false;
    delete task.progress;
    try {
      await retryTask();
      succeed(id);
    } catch (error) {
      task.status = "failed";
      task.statusMessage = String(error);
      task.retryable = true;
    }
  }

  return {
    delete: (id: string) => retries.delete(id),
    clear: () => retries.clear(),
    restoreFailed,
    setRetry,
    retry,
  };
}
