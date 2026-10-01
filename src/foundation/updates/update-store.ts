import { Channel } from "@tauri-apps/api/core";
import { error as logError } from "@tauri-apps/plugin-log";
import { defineStore } from "pinia";
import { computed, nextTick, shallowRef } from "vue";
import { toast } from "vue-sonner";

import type { UpdateSnapshot } from "../../shared/contracts/generated/UpdateSnapshot";
import { formatErrorDiagnostic } from "../errors/error-format";
import { i18n } from "../i18n/i18n";
import { invokeCommand } from "../ipc/invoke-command";
import { commitApplicationRestart } from "../lifecycle/window-close-coordinator";
import { useTaskStore } from "../tasks/task-store";
import { updateSnapshotSchema } from "./update-schema";
import { createUpdateTask } from "./update-task";

async function cancelDownload() {
  await invokeCommand("cancel_update_download");
}

export const useUpdateStore = defineStore("application-updates", () => {
  const snapshot = shallowRef<UpdateSnapshot | null>(null);
  const disconnected = shallowRef(false);
  const requesting = shallowRef(false);
  const preparing = shallowRef(false);
  const tasks = useTaskStore();
  const t = i18n.global.t;
  const mirrorTask = createUpdateTask(cancelDownload);
  let connecting: Promise<void> | undefined;
  let connected = false;
  let notifiedVersion: string | undefined;
  const phase = computed(() => snapshot.value?.phase);
  const blocking = computed(() => preparing.value || phase.value === "installing");
  const busy = computed(
    () =>
      requesting.value ||
      blocking.value ||
      phase.value === "checking" ||
      phase.value === "downloading" ||
      phase.value === "verifying",
  );

  function receive(value: unknown) {
    const next = updateSnapshotSchema.parse(value);
    if (snapshot.value && next.revision < snapshot.value.revision) return;
    const previousPhase = snapshot.value?.phase;
    snapshot.value = next;
    mirrorTask(next);
    if (next.phase === "ready" && previousPhase === "verifying") {
      toast.success(t("settings.updates.download.ready"), {
        action: {
          label: t("settings.updates.install.action"),
          onClick: () => {
            void install();
          },
        },
      });
    }
    if (next.phase === "available" && next.update && notifiedVersion !== next.update.version) {
      notifiedVersion = next.update.version;
      toast.info(t("settings.updates.available.title", { version: next.update.version }));
    }
  }

  function connect(): Promise<void> {
    if (connected) return Promise.resolve();
    connecting ??= observe().finally(() => {
      connecting = undefined;
    });
    return connecting;
  }

  async function observe() {
    const onEvent = new Channel<unknown>((value) => {
      try {
        receive(value);
      } catch (error) {
        disconnected.value = true;
        connected = false;
        void logError(`Invalid updater event: ${formatErrorDiagnostic(error)}`);
      }
    });
    try {
      receive(await invokeCommand<unknown>("observe_updates", { onEvent }));
      connected = true;
      disconnected.value = false;
    } catch (error) {
      disconnected.value = true;
      throw error;
    }
  }

  async function run(command: "check_for_update" | "download_update", message: string) {
    if (busy.value) return;
    requesting.value = true;
    try {
      await connect();
      const version = snapshot.value?.update?.version;
      receive(await invokeCommand<unknown>(command, command === "download_update" ? { version } : {}));
      if (command === "check_for_update" && !snapshot.value?.update) {
        toast.success(t("settings.updates.current.upToDate"));
      }
    } catch (error) {
      toast.error(message);
      void logError(`Application update failed: ${formatErrorDiagnostic(error)}`);
    } finally {
      requesting.value = false;
    }
  }

  async function cancel() {
    try {
      await cancelDownload();
    } catch (error) {
      toast.error(t("settings.updates.download.failed"));
      void logError(`Update cancellation failed: ${formatErrorDiagnostic(error)}`);
    }
  }

  async function install() {
    if (busy.value || phase.value !== "ready" || !snapshot.value?.update) return;
    if (tasks.running.length) {
      toast.error(t("settings.updates.install.runningTasks"));
      return;
    }
    const version = snapshot.value.update.version;
    preparing.value = true;
    try {
      await nextTick();
      await commitApplicationRestart(async () => {
        // Saves can start work; check again after every close hook has settled.
        if (tasks.running.length) throw new Error("Background tasks must finish before installing");
        await invokeCommand("install_update", { version });
      });
    } catch (error) {
      toast.error(t("settings.updates.install.failed"));
      void logError(`Application update install failed: ${formatErrorDiagnostic(error)}`);
    } finally {
      preparing.value = false;
    }
  }

  return {
    snapshot,
    disconnected,
    phase,
    busy,
    blocking,
    connect,
    install,
    cancel,
    check: () => run("check_for_update", t("settings.updates.check.failed")),
    download: () => run("download_update", t("settings.updates.download.failed")),
  };
});
