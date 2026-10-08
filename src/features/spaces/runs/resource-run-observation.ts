import { debounce } from "es-toolkit";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { i18n } from "../../../foundation/i18n/i18n";
import { invokeCommand } from "../../../foundation/ipc/invoke-command";
import { useTaskStore } from "../../../foundation/tasks/task-store";
import type { RunObservation } from "../contracts/generated/RunObservation";
import { useSpaceStore } from "../session/space-store";
import { workflowStateLabelKey } from "../workflows/workflow-state-label";
import { requestRunControl } from "./run-control";
import { observeRun } from "./run-observation";

type Listener = (run: RunObservation) => void;
const observations = new Map<string, Set<Listener>>();
export function observeResourceRun(run: RunObservation, title: string, onUpdate?: Listener) {
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
  tasks.restoreRunning({ id: run.id, title, startedAt: run.createdAt }, () => cancelResourceRun(run.id));
  tasks.setOpen(run.id, () => space.openSpace(run.homeSpaceId));
  const refreshCanvas = debounce(() => {
    if (space.activeSpaceId !== run.homeSpaceId) return;
    void space.refreshActiveSpace().catch((cause: unknown) => toast.error(getErrorMessage(cause)));
  }, 400);
  observeRun(run.id, {
    update: (next) => {
      for (const listener of listeners) listener(next);
      refreshCanvas();
      if (next.state === "succeeded") tasks.succeed(next.id);
      else if (next.state === "failed" || next.state === "completed_with_errors") {
        tasks.fail(next.id, i18n.global.t(workflowStateLabelKey(next.state)));
      } else if (next.state === "cancelled") tasks.cancelled(next.id);
      else tasks.setStatusMessage(next.id, i18n.global.t(workflowStateLabelKey(next.state)));
    },
    error: (cause) => tasks.fail(run.id, getErrorMessage(cause)),
    finished: () => {
      observations.delete(run.id);
      refreshCanvas.cancel();
      if (space.activeSpaceId === run.homeSpaceId) {
        void space.refreshActiveSpace().catch((cause: unknown) => toast.error(getErrorMessage(cause)));
      }
    },
  });
  return () => onUpdate && listeners.delete(onUpdate);
}

export async function restoreResourceRuns(title: string) {
  const runs = await invokeCommand<RunObservation[]>("plugin:spaces|list_active_resource_runs");
  for (const run of runs) observeResourceRun(run, title);
}

export function cancelResourceRun(runId: string): Promise<void> {
  return requestRunControl("cancel", runId);
}
