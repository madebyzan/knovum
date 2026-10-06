import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { error as logError } from "@tauri-apps/plugin-log";

import { formatErrorDiagnostic } from "../errors/error-format";
import { invokeCommand } from "../ipc/invoke-command";
import { isApplicationTransitioning, runApplicationTransition } from "./application-transition";

type WindowCloseCommit = () => Promise<void>;

const commits = new Set<WindowCloseCommit>();
let listener: Promise<() => void> | undefined;
let exitListener: Promise<() => void> | undefined;
let flushing: Promise<void> | undefined;

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
  flushing ??= commitRegisteredChanges().finally(() => {
    flushing = undefined;
  });
  return flushing;
}

async function commitRegisteredChanges(): Promise<void> {
  const outcomes = await Promise.allSettled([...commits].map((commit) => Promise.resolve().then(commit)));
  const failures = outcomes.filter((outcome) => outcome.status === "rejected");
  if (failures.length) {
    throw new AggregateError(
      failures.map((outcome) => outcome.reason),
      "Application changes could not be saved.",
    );
  }
}

async function attachExitRequestListener() {
  return listen("knovum://app-exit-requested", async () => {
    if (isApplicationTransitioning()) {
      await invokeCommand("plugin:desktop|decide_app_exit_request", { approved: false });
      return;
    }
    try {
      await runApplicationTransition(async () => {
        await flushWindowCloseCommits();
        await invokeCommand("plugin:desktop|decide_app_exit_request", { approved: true });
      });
    } catch (error) {
      await logError(`Application exit commit failed: ${formatErrorDiagnostic(error)}`);
      await invokeCommand("plugin:desktop|decide_app_exit_request", { approved: false });
    }
  });
}

async function attachWindowCloseListener() {
  return getCurrentWindow().onCloseRequested(async (event) => {
    event.preventDefault();
    if (isApplicationTransitioning()) return;
    try {
      await runApplicationTransition(async () => {
        await flushWindowCloseCommits();
        await getCurrentWindow().hide();
      });
    } catch (error) {
      await logError(`Window close commit failed: ${formatErrorDiagnostic(error)}`);
    }
  });
}
