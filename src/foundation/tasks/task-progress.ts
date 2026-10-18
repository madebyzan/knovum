import { clamp } from "es-toolkit";
import type { Ref } from "vue";

import type { TaskReference } from "../../shared/contracts/task-reference";
import type { AppTask } from "./task-model";

export function createTaskProgress(tasks: Ref<AppTask[]>) {
  function updateRunning(
    taskId: string,
    changes: Partial<Pick<AppTask, "progress" | "statusMessage" | "title" | "subject">>,
  ) {
    const task = tasks.value.find((candidate) => candidate.id === taskId);
    if (task?.status === "running") Object.assign(task, changes);
  }

  return {
    indeterminate(taskId: string, statusMessage?: string) {
      const task = tasks.value.find((candidate) => candidate.id === taskId);
      if (!task || task.status !== "running") return;
      delete task.progress;
      if (statusMessage !== undefined) task.statusMessage = statusMessage;
    },
    progress(taskId: string, value: number, statusMessage?: string) {
      updateRunning(taskId, {
        progress: clamp(value, 0, 100),
        ...(statusMessage === undefined ? {} : { statusMessage }),
      });
    },
    setStatusMessage(taskId: string, statusMessage: string) {
      updateRunning(taskId, { statusMessage });
    },
    /** Names what the work is being done to, once the work knows. */
    describeSubject: (taskId: string, subject: TaskReference) => updateRunning(taskId, { subject }),
    title: (taskId: string, value: string) => updateRunning(taskId, { title: value }),
  };
}
