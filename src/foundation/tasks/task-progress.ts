import { clamp } from "es-toolkit";
import type { Ref } from "vue";

import type { AppTask } from "./task-model";

export function createTaskProgress(tasks: Ref<AppTask[]>) {
  function updateRunning(id: string, changes: Partial<Pick<AppTask, "progress" | "detail" | "title">>) {
    const task = tasks.value.find((candidate) => candidate.id === id);
    if (task?.status === "running") Object.assign(task, changes);
  }

  return {
    progress(id: string, value: number, taskDetail?: string) {
      updateRunning(id, {
        progress: clamp(value, 0, 100),
        ...(taskDetail === undefined ? {} : { detail: taskDetail }),
      });
    },
    detail: (id: string, value: string) => updateRunning(id, { detail: value }),
    title: (id: string, value: string) => updateRunning(id, { title: value }),
  };
}
