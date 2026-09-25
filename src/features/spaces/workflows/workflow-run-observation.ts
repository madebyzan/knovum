import { Channel, invoke } from "@tauri-apps/api/core";
import { shallowRef } from "vue";
import { toast } from "vue-sonner";

import { i18n } from "../../../foundation/i18n/i18n";
import { useTaskStore } from "../../../foundation/tasks/task-store";
import type { RunSummary } from "../contracts/generated/RunSummary";
import { useSpaceStore } from "../session/space-store";

type Listener = (run: RunSummary) => void;
const observations = new Map<string, Set<Listener>>();
export const reviewRunId = shallowRef("");

export function closeWorkflowReview() {
  reviewRunId.value = "";
}

export function observeWorkflowRun(run: RunSummary, title: string, onUpdate?: Listener) {
  const existing = observations.get(run.id);
  if (existing) {
    if (onUpdate) existing.add(onUpdate);
    return () => onUpdate && existing.delete(onUpdate);
  }
  const listeners = new Set<Listener>();
  if (onUpdate) listeners.add(onUpdate);
  observations.set(run.id, listeners);
  const tasks = useTaskStore();
  const space = useSpaceStore();
  tasks.restoreRunning({ id: run.id, title, startedAt: run.createdAt }, () => cancelWorkflowRun(run.id));
  function attachNavigation(current: RunSummary) {
    tasks.setOpen(run.id, async () => {
      await space.openSpace(run.homeSpaceId);
      if (current.waitReason === "content_review") reviewRunId.value = run.id;
    });
  }
  attachNavigation(run);
  let refreshTimer: ReturnType<typeof setTimeout> | undefined;
  function refreshCanvas() {
    if (space.activeSpaceId !== run.homeSpaceId) return;
    if (refreshTimer) clearTimeout(refreshTimer);
    refreshTimer = setTimeout(() => {
      refreshTimer = undefined;
      void space.refreshActiveSpace().catch((cause: unknown) => toast.error(String(cause)));
    }, 400);
  }
  const updates = new Channel<RunSummary>();
  // Tauri Channel is an IPC receiver, not an EventTarget.
  // oxlint-disable-next-line unicorn/prefer-add-event-listener
  updates.onmessage = (next) => {
    for (const listener of listeners) listener(next);
    attachNavigation(next);
    refreshCanvas();
    if (next.state === "succeeded") tasks.succeed(next.id);
    else if (next.state === "failed" || next.state === "completed_with_errors") {
      tasks.fail(next.id, i18n.global.t(`spaces.creation.workflowStates.${next.state}`));
    } else if (next.state === "cancelled") tasks.cancelled(next.id);
    else if (next.waitReason === "content_review") {
      tasks.detail(next.id, i18n.global.t("spaces.creation.workflowReviewRequired"));
    } else tasks.detail(next.id, i18n.global.t(`spaces.creation.workflowStates.${next.state}`));
    if (reviewRunId.value === next.id && next.waitReason !== "content_review") closeWorkflowReview();
  };
  void invoke("plugin:spaces|observe_run", { runId: run.id, updates })
    .catch((cause: unknown) => tasks.fail(run.id, String(cause)))
    .finally(() => {
      observations.delete(run.id);
      if (refreshTimer) clearTimeout(refreshTimer);
      if (space.activeSpaceId === run.homeSpaceId) {
        void space.refreshActiveSpace().catch((cause: unknown) => toast.error(String(cause)));
      }
    });
  return () => onUpdate && listeners.delete(onUpdate);
}

export async function restoreWorkflowRuns(title: string) {
  const runs = await invoke<RunSummary[]>("plugin:spaces|list_active_workflow_runs");
  for (const run of runs) observeWorkflowRun(run, title);
}

export function cancelWorkflowRun(runId: string): Promise<void> {
  return invoke("plugin:spaces|cancel_run", { runId });
}
