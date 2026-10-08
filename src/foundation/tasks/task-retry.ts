import type { Ref } from "vue";

import { getErrorMessage } from "../errors/error-format";
import type { AppTask } from "./task-model";

export type RetryTask = () => Promise<void>;
export interface RestoreFailedOptions {
  settlement?: "automatic" | "external";
}

interface RetryDescriptor {
  run: RetryTask;
  settlement: "automatic" | "external";
}

interface TaskRetryOptions {
  tasks: Ref<AppTask[]>;
  succeed: (id: string) => void;
  trimHistory: () => void;
  externallySettled: Set<string>;
}

export function createTaskRetryController({
  tasks,
  succeed,
  trimHistory,
  externallySettled,
}: TaskRetryOptions) {
  const retries = new Map<string, RetryDescriptor>();
  const observationRetries = new Map<string, RetryTask>();

  function restoreFailed(
    task: Omit<AppTask, "status">,
    retryTask: RetryTask,
    options: RestoreFailedOptions = {},
  ) {
    if (tasks.value.some(({ id }) => id === task.id)) return false;
    tasks.value.unshift({ ...task, status: "failed", retryable: true });
    retries.set(task.id, {
      run: retryTask,
      settlement: options.settlement ?? "automatic",
    });
    trimHistory();
    return true;
  }

  function setRetry(id: string, retryTask: RetryTask) {
    const task = tasks.value.find((candidate) => candidate.id === id);
    if (!task || task.status !== "failed") return;
    retries.set(id, { run: retryTask, settlement: "automatic" });
    task.retryable = true;
  }

  function markObservationUnavailable(id: string, message: string, retryTask: RetryTask) {
    const task = tasks.value.find((candidate) => candidate.id === id);
    if (!task || (task.status !== "running" && task.status !== "attention")) return;
    task.status = "attention";
    task.statusMessage = message;
    task.retryable = true;
    observationRetries.set(id, retryTask);
  }

  function resumeObservation(id: string) {
    const task = tasks.value.find((candidate) => candidate.id === id);
    if (!task || task.status !== "attention") return;
    task.status = "running";
    task.retryable = false;
    observationRetries.delete(id);
  }

  async function retryObservation(id: string, task: AppTask) {
    const observe = observationRetries.get(id);
    if (!observe || !task.retryable) return;
    resumeObservation(id);
    try {
      await observe();
    } catch (error) {
      markObservationUnavailable(id, getErrorMessage(error), observe);
    }
  }

  function failRetry(id: string, error: unknown) {
    const current = tasks.value.find((candidate) => candidate.id === id);
    if (current?.status === "failed") {
      current.retryable = true;
      return;
    }
    if (current?.status !== "running") return;
    externallySettled.delete(id);
    current.status = "failed";
    current.statusMessage = getErrorMessage(error);
    current.retryable = true;
  }

  async function retry(id: string) {
    const task = tasks.value.find((candidate) => candidate.id === id);
    if (task?.status === "attention") return retryObservation(id, task);
    const retryDescriptor = retries.get(id);
    if (!task || task.status !== "failed" || !retryDescriptor) return;
    task.status = "running";
    task.retryable = false;
    delete task.progress;
    if (retryDescriptor.settlement === "external") externallySettled.add(id);
    try {
      await retryDescriptor.run();
      succeed(id);
    } catch (error) {
      failRetry(id, error);
    }
  }

  return {
    delete: (id: string) => {
      retries.delete(id);
      observationRetries.delete(id);
    },
    clear: () => {
      retries.clear();
      observationRetries.clear();
    },
    restoreFailed,
    setRetry,
    markObservationUnavailable,
    resumeObservation,
    retry,
  };
}
