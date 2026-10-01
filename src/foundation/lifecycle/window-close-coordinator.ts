import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { error as logError } from "@tauri-apps/plugin-log";

import { formatErrorDiagnostic } from "../errors/error-format";
import { invokeCommand } from "../ipc/invoke-command";

type WindowCloseCommit = () => Promise<void>;

const commits = new Set<WindowCloseCommit>();
let listener: Promise<() => void> | undefined;
let exitListener: Promise<() => void> | undefined;
let flushing: Promise<void> | undefined;
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

// All callers share the same save barrier. A failed save must not release the
// install/exit guard while another registered save is still running.
export function flushWindowCloseCommits(): Promise<void> {
  flushing ??= Promise.allSettled([...commits].map((commit) => Promise.resolve().then(commit)))
    .then((results) => {
      const failures = results.filter((result) => result.status === "rejected");
      if (failures.length) throw new AggregateError(failures.map((result) => result.reason), "Application changes could not be saved.");
    })
    .finally(() => { flushing = undefined; });
  return flushing;
}

async function attachExitRequestListener() {
  return listen("knovum://app-exit-requested", async () => {
    if (exiting) return;
    exiting = true;
    try {
      await flushWindowCloseCommits();
      await invokeCommand("plugin:desktop|decide_app_exit_request", { approved: true });
    } catch (error) {
      await logError(`Application exit commit failed: ${formatErrorDiagnostic(error)}`);
      await invokeCommand("plugin:desktop|decide_app_exit_request", { approved: false });
    } finally {
      exiting = false;
    }
  });
}

async function attachWindowCloseListener() {
  return getCurrentWindow().onCloseRequested(async (event) => {
    event.preventDefault();
    if (closing) return;
    closing = true;
    try {
      await flushWindowCloseCommits();
      await getCurrentWindow().hide();
    } catch (error) {
      await logError(`Window close commit failed: ${formatErrorDiagnostic(error)}`);
    } finally {
      closing = false;
    }
  });
}
