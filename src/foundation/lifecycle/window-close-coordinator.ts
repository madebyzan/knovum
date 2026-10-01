import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { error as logError } from "@tauri-apps/plugin-log";

import { formatErrorDiagnostic } from "../errors/error-format";
import { invokeCommand } from "../ipc/invoke-command";

type WindowCloseCommit = () => Promise<void>;

const commits = new Set<WindowCloseCommit>();
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

export function registerWindowCloseCommit(commit: WindowCloseCommit) {
  commits.add(commit);
  void initializeExitCoordination().catch((error) =>
    logError(`Application close coordinator initialization failed: ${formatErrorDiagnostic(error)}`),
  );
  return () => commits.delete(commit);
}

async function attachExitRequestListener() {
  return listen("knovum://app-exit-requested", async () => {
    if (exiting) return;
    exiting = true;
    try {
      await commitAll();
      await invokeCommand("plugin:desktop|decide_app_exit_request", { approved: true });
    } catch (error) {
      await logError(`Application exit commit failed: ${formatErrorDiagnostic(error)}`);
      await invokeCommand("plugin:desktop|decide_app_exit_request", { approved: false });
    } finally {
      exiting = false;
    }
  });
}

function commitAll() {
  return Promise.all([...commits].map((commit) => commit())).then(() => undefined);
}

async function attachWindowCloseListener() {
  return getCurrentWindow().onCloseRequested(async (event) => {
    event.preventDefault();
    if (closing) return;
    closing = true;
    try {
      await commitAll();
      await getCurrentWindow().hide();
    } catch (error) {
      await logError(`Window close commit failed: ${formatErrorDiagnostic(error)}`);
    } finally {
      closing = false;
    }
  });
}
