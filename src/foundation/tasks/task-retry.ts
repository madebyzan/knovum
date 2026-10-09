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
  succeed: (taskId: string) => void;
  trimHistory: () => void;
  externallySettledTaskIds: Set<string>;
}

export function createTaskRetryController({
  tasks,
  succeed,
  trimHistory,
  externallySettledTaskIds,
}: TaskRetryOptions) {
  const retryDescriptorByTaskId = new Map<string, RetryDescriptor>();
  const observationRetryByTaskId = new Map<string, RetryTask>();

  function restoreFailed(
    task: Omit<AppTask, "status">,
    retryTask: RetryTask,
    options: RestoreFailedOptions = {},
  ) {
    if (tasks.value.some(({ id }) => id === task.id)) return false;
    tasks.value.unshift({ ...task, status: "failed", retryable: true });
    retryDescriptorByTaskId.set(task.id, {
      run: retryTask,
      settlement: options.settlement ?? "automatic",
    });
    trimHistory();
    return true;
  }

  function setRetry(taskId: string, retryTask: RetryTask) {
    const task = tasks.value.find((candidate) => candidate.id === taskId);
    if (!task || task.status !== "failed") return;
    retryDescriptorByTaskId.set(taskId, { run: retryTask, settlement: "automatic" });
    task.retryable = true;
  }

  function markObservationUnavailable(taskId: string, message: string, retryTask: RetryTask) {
    const task = tasks.value.find((candidate) => candidate.id === taskId);
    if (!task || (task.status !== "running" && task.status !== "attention")) return;
    task.status = "attention";
    task.statusMessage = message;
    task.retryable = true;
    observationRetryByTaskId.set(taskId, retryTask);
  }

  function resumeObservation(taskId: string) {
    const task = tasks.value.find((candidate) => candidate.id === taskId);
    if (!task || task.status !== "attention") return;
    task.status = "running";
    task.retryable = false;
    observationRetryByTaskId.delete(taskId);
  }

  async function retryObservation(taskId: string, task: AppTask) {
    const observe = observationRetryByTaskId.get(taskId);
    if (!observe || !task.retryable) return;
    resumeObservation(taskId);
    try {
      await observe();
    } catch (error) {
      markObservationUnavailable(taskId, getErrorMessage(error), observe);
    }
  }

  function failRetry(taskId: string, error: unknown) {
    const current = tasks.value.find((candidate) => candidate.id === taskId);
    if (current?.status === "failed") {
      current.retryable = true;
      return;
    }
    if (current?.status !== "running") return;
    externallySettledTaskIds.delete(taskId);
    current.status = "failed";
    current.statusMessage = getErrorMessage(error);
    current.retryable = true;
  }

  async function retry(taskId: string) {
    const task = tasks.value.find((candidate) => candidate.id === taskId);
    if (task?.status === "attention") return retryObservation(taskId, task);
    const retryDescriptor = retryDescriptorByTaskId.get(taskId);
    if (!task || task.status !== "failed" || !retryDescriptor) return;
    task.status = "running";
    task.retryable = false;
    delete task.progress;
    if (retryDescriptor.settlement === "external") externallySettledTaskIds.add(taskId);
    try {
      await retryDescriptor.run();
      succeed(taskId);
    } catch (error) {
      failRetry(taskId, error);
    }
  }

  return {
    delete: (taskId: string) => {
      retryDescriptorByTaskId.delete(taskId);
      observationRetryByTaskId.delete(taskId);
    },
    clear: () => {
      retryDescriptorByTaskId.clear();
      observationRetryByTaskId.clear();
    },
    restoreFailed,
    setRetry,
    markObservationUnavailable,
    resumeObservation,
    retry,
  };
}
