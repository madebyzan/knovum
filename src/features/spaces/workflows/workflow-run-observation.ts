import { debounce } from "es-toolkit";
import { shallowRef } from "vue";
import { toast } from "vue-sonner";

import { i18n } from "../../../foundation/i18n/i18n";
import { invokeCommand } from "../../../foundation/ipc/invoke-command";
import { useTaskStore } from "../../../foundation/tasks/task-store";
import type { RunObservation } from "../contracts/generated/RunObservation";
import { observeRun } from "../runs/run-observation";
import { useSpaceStore } from "../session/space-store";
import { workflowStateLabelKey } from "./workflow-state-label";

type Listener = (run: RunObservation) => void;
const observations = new Map<string, Set<Listener>>();
export const reviewRunId = shallowRef("");

export function closeWorkflowReview() {
  reviewRunId.value = "";
}

export function observeWorkflowRun(run: RunObservation, title: string, onUpdate?: Listener) {
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
  function attachNavigation(current: RunObservation) {
    tasks.setOpen(run.id, async () => {
      await space.openSpace(run.homeSpaceId);
      if (current.waitReason === "content_review") reviewRunId.value = run.id;
    });
  }
  attachNavigation(run);
  const refreshCanvas = debounce(() => {
    if (space.activeSpaceId !== run.homeSpaceId) return;
    void space.refreshActiveSpace().catch((cause: unknown) => toast.error(String(cause)));
  }, 400);
  observeRun(run.id, {
    update: (next) => {
      for (const listener of listeners) listener(next);
      attachNavigation(next);
      refreshCanvas();
      if (next.state === "succeeded") tasks.succeed(next.id);
      else if (next.state === "failed" || next.state === "completed_with_errors") {
        tasks.fail(next.id, i18n.global.t(workflowStateLabelKey(next.state)));
      } else if (next.state === "cancelled") tasks.cancelled(next.id);
      else if (next.waitReason === "content_review") {
        tasks.setStatusMessage(next.id, i18n.global.t("spaces.creation.workflow.review.required"));
      } else tasks.setStatusMessage(next.id, i18n.global.t(workflowStateLabelKey(next.state)));
      if (reviewRunId.value === next.id && next.waitReason !== "content_review") closeWorkflowReview();
    },
    error: (cause) => tasks.fail(run.id, String(cause)),
    finished: () => {
      observations.delete(run.id);
      refreshCanvas.cancel();
      if (space.activeSpaceId === run.homeSpaceId) {
        void space.refreshActiveSpace().catch((cause: unknown) => toast.error(String(cause)));
      }
    },
  });
  return () => onUpdate && listeners.delete(onUpdate);
}

export async function restoreWorkflowRuns(title: string) {
  const runs = await invokeCommand<RunObservation[]>("plugin:spaces|list_active_workflow_runs");
  for (const run of runs) observeWorkflowRun(run, title);
}

export function cancelWorkflowRun(runId: string): Promise<void> {
  return invokeCommand("plugin:spaces|cancel_run", { runId });
}
