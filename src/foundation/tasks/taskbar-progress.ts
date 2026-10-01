import { getCurrentWindow, ProgressBarStatus } from "@tauri-apps/api/window";
import { error as logError } from "@tauri-apps/plugin-log";
import { meanBy } from "es-toolkit";
import type { Pinia } from "pinia";
import { watch } from "vue";

import { formatErrorDiagnostic } from "../errors/error-format";
import { useTaskStore } from "./task-store";

interface RunningTaskSnapshot {
  progress?: number | undefined;
}

async function updateTaskbar(tasks: readonly RunningTaskSnapshot[]): Promise<void> {
  const window = getCurrentWindow();
  if (tasks.length === 0) {
    await window.setProgressBar({ status: ProgressBarStatus.None });
    return;
  }

  const determinate = tasks.filter(
    (task): task is RunningTaskSnapshot & { progress: number } => task.progress !== undefined,
  );
  if (determinate.length !== tasks.length) {
    await window.setProgressBar({ status: ProgressBarStatus.Indeterminate });
    return;
  }

  await window.setProgressBar({
    status: ProgressBarStatus.Normal,
    progress: Math.round(meanBy(determinate, (task) => task.progress)),
  });
}

export function installTaskbarProgress(pinia: Pinia): void {
  const taskStore = useTaskStore(pinia);
  watch(
    () => taskStore.running.map(({ progress }) => ({ progress })),
    (tasks) => {
      void updateTaskbar(tasks).catch((reason) =>
        logError(`Failed to update taskbar progress: ${formatErrorDiagnostic(reason)}`),
      );
    },
    { deep: true, immediate: true },
  );
}
