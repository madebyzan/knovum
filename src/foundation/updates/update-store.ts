import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { error as logError } from "@tauri-apps/plugin-log";
import { defineStore } from "pinia";
import { computed, onScopeDispose, shallowRef } from "vue";

import type { UpdateSnapshot } from "../../shared/contracts/generated/UpdateSnapshot";
import { formatErrorDiagnostic } from "../errors/error-format";
import { invokeCommand } from "../ipc/invoke-command";
import { flushWindowCloseCommits } from "../lifecycle/window-close-coordinator";
import { useTaskStore } from "../tasks/task-store";
import { updateSnapshotSchema } from "./update-schema";

export const applicationUpdateTaskId = "application-update";

export const useUpdateStore = defineStore("application-update", () => {
  const snapshot = shallowRef<UpdateSnapshot | null>(null);
  const requestPending = shallowRef(false);
  const preparingInstall = shallowRef(false);
  const tasks = useTaskStore();
  let initialization: Promise<void> | undefined;
  let unlisten: UnlistenFn | undefined;
  let disposed = false;
  const state = computed(() => snapshot.value?.state ?? null);
  const blockedByTasks = computed(() => tasks.running.some((task) => task.id !== applicationUpdateTaskId));
  const installing = computed(() => preparingInstall.value || state.value?.status === "installing");
  const canInstall = computed(() => state.value?.status === "ready" && !blockedByTasks.value && !requestPending.value && !installing.value);

  function accept(value: unknown) {
    const next = updateSnapshotSchema.parse(value);
    if (!snapshot.value || next.revision > snapshot.value.revision) snapshot.value = next;
  }

  async function connect() {
    const stop = await listen<unknown>("knovum://update-state", (event) => {
      try {
        accept(event.payload);
      } catch (error) {
        void logError(`Invalid updater event: ${formatErrorDiagnostic(error)}`);
      }
    });
    try {
      const initial = await invokeCommand<unknown>("get_update_state");
      if (disposed) { stop(); return; }
      accept(initial);
      unlisten = stop;
    } catch (error) {
      stop();
      throw error;
    }
  }

  function initialize() {
    initialization ??= connect().catch((error) => {
      initialization = undefined;
      throw error;
    });
    return initialization;
  }

  async function request(command: string, payload?: Record<string, unknown>) {
    if (requestPending.value || installing.value) throw new Error("An updater request is already running.");
    requestPending.value = true;
    try {
      await initialize();
      const response = await invokeCommand<unknown>(command, payload);
      accept(response);
    } finally {
      requestPending.value = false;
    }
  }

  async function download() {
    if (state.value?.status !== "available") throw new Error("No selected update is available.");
    await request("download_update", { version: state.value.release.version });
  }

  async function install() {
    if (!canInstall.value || state.value?.status !== "ready") throw new Error("Finish background tasks before installing the update.");
    const version = state.value.release.version;
    preparingInstall.value = true;
    try {
      await flushWindowCloseCommits();
      if (blockedByTasks.value) throw new Error("A background task started while preparing the update.");
      accept(await invokeCommand<unknown>("install_update", { version }));
    } finally {
      preparingInstall.value = false;
    }
  }

  onScopeDispose(() => { disposed = true; unlisten?.(); });

  return {
    state, snapshot, requestPending, installing, blockedByTasks, canInstall, initialize, download, install,
    check: () => request("check_for_update"),
    cancelDownload: () => invokeCommand<void>("cancel_update_download"),
  };
});
