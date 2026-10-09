import { error as logError } from "@tauri-apps/plugin-log";
import type { Pinia } from "pinia";
import { watch } from "vue";

import type { DesktopTask } from "../../shared/contracts/generated/DesktopTask";
import { formatErrorDiagnostic } from "../errors/error-format";
import { invokeCommand } from "../ipc/invoke-command";
import { useTaskStore } from "./task-store";

export function installSessionTaskProjection(pinia: Pinia): void {
  const taskStore = useTaskStore(pinia);
  let publication = Promise.resolve();
  watch(
    () =>
      taskStore.tasks
        .filter((task) => !task.durable)
        .map<DesktopTask>(({ id, status, progress, startedAt }) => ({
          id,
          state: status,
          progress: progress === undefined ? null : Math.round(progress),
          startedAt,
        })),
    (tasks) => {
      publication = publication
        .then(() => invokeCommand<void>("plugin:desktop|project_session_tasks", { tasks }))
        .catch((reason) => logError(`Desktop task projection failed: ${formatErrorDiagnostic(reason)}`));
    },
    { deep: true, immediate: true },
  );
}
