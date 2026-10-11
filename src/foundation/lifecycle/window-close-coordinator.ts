import { listen } from "@tauri-apps/api/event";
import { error as logError } from "@tauri-apps/plugin-log";
import { toast } from "vue-sonner";

import {
  ApplicationLeaveCancelled,
  isApplicationLeaveCancelled,
} from "../../shared/contracts/application-leave";
import type { AppExitRequest } from "../../shared/contracts/generated/AppExitRequest";
import { formatErrorDiagnostic, getErrorMessage } from "../errors/error-format";
import { i18n } from "../i18n/instance";
import { invokeCommand } from "../ipc/invoke-command";
import { isApplicationTransitioning, runApplicationTransition } from "./application-transition";
import { startRequestHeartbeat } from "./request-heartbeat";
import { attachWindowCloseListener } from "./window-close-request";

type WindowCloseCommit = () => Promise<void>;

const commits = new Set<WindowCloseCommit>();
let listener: Promise<() => void> | undefined;
let exitListener: Promise<() => void> | undefined;
let flushing: Promise<void> | undefined;
let activeExitRequest: number | undefined;

export async function initializeExitCoordination() {
  listener ??= attachWindowCloseListener({
    flush: flushWindowCloseCommits,
    reportFailure: reportCloseFailure,
  });
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
  const errors: unknown[] = [];
  for (const commit of commits) {
    try {
      // Each participant may open a decision dialog; settle it before asking the next owner.
      // oxlint-disable-next-line no-await-in-loop
      await commit();
    } catch (error) {
      errors.push(error);
      if (isApplicationLeaveCancelled(error)) break;
    }
  }
  const realFailures = errors.filter((error) => !isApplicationLeaveCancelled(error));
  if (realFailures.length) {
    throw new AggregateError(realFailures, "Application changes could not be saved.");
  }
  if (errors.length) throw new ApplicationLeaveCancelled();
}

async function attachExitRequestListener() {
  return listen<AppExitRequest>("knovum://app-exit-requested", ({ payload }) =>
    handleExitRequest(payload.requestId),
  );
}

async function handleExitRequest(requestId: number) {
  if (activeExitRequest === requestId) return;
  if (activeExitRequest !== undefined) {
    await decideExitRequest(requestId, false);
    return;
  }
  activeExitRequest = requestId;
  let stopHeartbeat: (() => void) | undefined;
  try {
    if (!(await acknowledgeExitRequest(requestId))) return;
    stopHeartbeat = startRequestHeartbeat(() => acknowledgeExitRequest(requestId));
    if (isApplicationTransitioning()) {
      await decideExitRequest(requestId, false);
      return;
    }
    try {
      await runApplicationTransition(async () => {
        await flushWindowCloseCommits();
        await decideExitRequest(requestId, true);
      });
    } catch (error) {
      if (!isApplicationLeaveCancelled(error)) {
        await logError(`Application exit commit failed: ${formatErrorDiagnostic(error)}`);
        reportCloseFailure(error);
      }
      await decideExitRequest(requestId, false);
    }
  } finally {
    stopHeartbeat?.();
    activeExitRequest = undefined;
  }
}

async function acknowledgeExitRequest(requestId: number) {
  try {
    await invokeCommand("plugin:desktop|acknowledge_app_exit_request", { requestId });
    return true;
  } catch (error) {
    await logError(`Application exit acknowledgement failed: ${formatErrorDiagnostic(error)}`);
    return false;
  }
}

async function decideExitRequest(requestId: number, approved: boolean) {
  try {
    await invokeCommand("plugin:desktop|decide_app_exit_request", { requestId, approved });
  } catch (error) {
    await logError(`Application exit decision failed: ${formatErrorDiagnostic(error)}`);
  }
}

function reportCloseFailure(error: unknown, retry?: () => void) {
  toast.error(`${i18n.global.t("common.closeSaveFailed")}: ${getErrorMessage(error)}`, {
    id: "window-close-save-failed",
    ...(retry ? { action: { label: i18n.global.t("common.retry"), onClick: retry } } : {}),
  });
}
