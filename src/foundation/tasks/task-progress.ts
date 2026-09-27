import { clamp } from "es-toolkit";
import type { Ref } from "vue";

import type { AppTask } from "./task-model";

export function createTaskProgress(tasks: Ref<AppTask[]>) {
  function updateRunning(
    id: string,
    changes: Partial<Pick<AppTask, "progress" | "statusMessage" | "title">>,
  ) {
    const task = tasks.value.find((candidate) => candidate.id === id);
    if (task?.status === "running") Object.assign(task, changes);
  }

  return {
    indeterminate(id: string, statusMessage?: string) {
      const task = tasks.value.find((candidate) => candidate.id === id);
      if (!task || task.status !== "running") return;
      delete task.progress;
      if (statusMessage !== undefined) task.statusMessage = statusMessage;
    },
    progress(id: string, value: number, statusMessage?: string) {
      updateRunning(id, {
        progress: clamp(value, 0, 100),
        ...(statusMessage === undefined ? {} : { statusMessage }),
      });
    },
    setStatusMessage(id: string, statusMessage: string) {
      updateRunning(id, { statusMessage });
    },
    title: (id: string, value: string) => updateRunning(id, { title: value }),
  };
}
