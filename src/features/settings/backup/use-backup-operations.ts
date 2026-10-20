import { open, save } from "@tauri-apps/plugin-dialog";
import { computed, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import { invokeCommand } from "../../../foundation/ipc/invoke-command";
import { runApplicationTransition } from "../../../foundation/lifecycle/application-transition";
import { flushWindowCloseCommits } from "../../../foundation/lifecycle/window-close-coordinator";
import { useTaskStore } from "../../../foundation/tasks/task-store";
import { confirmAction } from "../../../shared/ui/confirm-action";
import { LOCALE_KEYS_BY_DATA_MAINTENANCE_OPERATION } from "./data-maintenance-locale";

const FILTER = [{ name: "Knovum backup", extensions: ["knovum-backup"] }];

export function useBackupOperations() {
  const { t } = useI18n();
  const busy = shallowRef<"backup" | "restore">();
  const tasks = useTaskStore();
  const blockedByTasks = computed(() => tasks.running.length > 0);

  async function backup() {
    const destination = await save({ defaultPath: "Knovum Backup.knovum-backup", filters: FILTER });
    if (!destination || !(await confirm("backup"))) return;
    await restartFor("backup", destination);
  }

  async function restore() {
    const source = await open({ multiple: false, directory: false, filters: FILTER });
    if (typeof source !== "string" || !(await confirm("restore"))) return;
    await restartFor("restore", source);
  }

  function confirm(operation: "backup" | "restore") {
    const keys = LOCALE_KEYS_BY_DATA_MAINTENANCE_OPERATION[operation];
    return confirmAction({
      title: t(keys.confirmTitle),
      description: t(keys.confirmDescription),
      confirmLabel: t(keys.action),
      danger: operation === "restore",
    });
  }

  async function restartFor(operation: "backup" | "restore", path: string) {
    assertTasksSettled();
    busy.value = operation;
    try {
      await runApplicationTransition(async () => {
        await flushWindowCloseCommits();
        assertTasksSettled();
        const command = operation === "backup" ? "schedule_backup" : "schedule_restore";
        const argument = operation === "backup" ? { destination: path } : { source: path };
        await invokeCommand<void>(`plugin:backup|${command}`, argument);
      });
    } finally {
      busy.value = undefined;
    }
  }

  function assertTasksSettled() {
    if (blockedByTasks.value) throw new Error(t("settings.data.tasksActive"));
  }

  return { busy, blockedByTasks, backup, restore };
}
