import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { error as logError } from "@tauri-apps/plugin-log";

import { formatErrorDiagnostic } from "../errors/error-format";
import { invokeCommand } from "../ipc/invoke-command";
import { CloseCommitCoordinator } from "./close-commits";

const coordinator = new CloseCommitCoordinator();
let listener: Promise<() => void> | undefined;
let exitListener: Promise<() => void> | undefined;
let closing = false;
let exiting = false;

export async function initializeExitCoordination() {
  listener ??= attachWindowCloseListener();
  exitListener ??= attachExitRequestListener();
  await Promise.all([listener, exitListener]);
  await invokeCommand("plugin:desktop|mark_exit_coordinator_ready");
}

export function registerWindowCloseCommit(commit: () => Promise<void>) {
  const unregister = coordinator.register(commit);
  void initializeExitCoordination().catch((error) =>
    logError(`Application close coordinator initialization failed: ${formatErrorDiagnostic(error)}`),
  );
  return unregister;
}

async function attachExitRequestListener() {
  return listen("knovum://app-exit-requested", async () => {
    if (coordinator.restarting) {
      await invokeCommand("plugin:desktop|decide_app_exit_request", { approved: false });
      return;
    }
    if (exiting) return;
    exiting = true;
    try {
      await coordinator.flush();
      await invokeCommand("plugin:desktop|decide_app_exit_request", { approved: true });
    } catch (error) {
      await logError(`Application exit commit failed: ${formatErrorDiagnostic(error)}`);
      await invokeCommand("plugin:desktop|decide_app_exit_request", { approved: false });
    } finally {
      exiting = false;
    }
  });
}

export function commitApplicationRestart(install: () => Promise<void>): Promise<void> {
  if (closing || exiting) return Promise.reject(new Error("Application close is already in progress"));
  return coordinator.restart(install);
}

async function attachWindowCloseListener() {
  return getCurrentWindow().onCloseRequested(async (event) => {
    event.preventDefault();
    if (closing || coordinator.restarting) return;
    closing = true;
    try {
      await coordinator.flush();
      await getCurrentWindow().hide();
    } catch (error) {
      await logError(`Window close commit failed: ${formatErrorDiagnostic(error)}`);
    } finally {
      closing = false;
    }
  });
}

export function isApplicationRestarting() {
  return coordinator.restarting;
}
