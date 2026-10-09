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
const listenersByRunId = new Map<string, Set<Listener>>();
let openVoiceLibrary: (() => Promise<void>) | undefined;

export function registerVoiceLibraryRunNavigation(handler: () => Promise<void>) {
  openVoiceLibrary = handler;
  return () => {
    if (openVoiceLibrary === handler) openVoiceLibrary = undefined;
  };
}

async function openResourceRun(run: RunObservation) {
  if (run.scope.kind === "space") {
    await useSpaceStore().openSpace(run.scope.spaceId);
    return;
  }
  if (!openVoiceLibrary) throw new Error("Voice library navigation is unavailable");
  await openVoiceLibrary();
}

function refreshRunSpace(run: RunObservation) {
  if (run.scope.kind !== "space") return;
  const space = useSpaceStore();
  if (space.activeSpaceId !== run.scope.spaceId) return;
  void space.refreshActiveSpace().catch((cause: unknown) => toast.error(getErrorMessage(cause)));
}

export function observeResourceRun(run: RunObservation, title: string, onUpdate?: Listener) {
  const existing = listenersByRunId.get(run.id);
  if (existing) {
    if (onUpdate) existing.add(onUpdate);
    return () => {
      if (onUpdate) existing.delete(onUpdate);
    };
  }
  const listeners = new Set<Listener>();
  if (onUpdate) listeners.add(onUpdate);
  listenersByRunId.set(run.id, listeners);
  const tasks = useTaskStore();
  tasks.restoreRunning({ id: run.id, title, startedAt: run.createdAt }, () => cancelResourceRun(run.id));
  tasks.setOpen(run.id, () => openResourceRun(run));
  const refreshCanvas = debounce(() => refreshRunSpace(run), 400);
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
      listenersByRunId.delete(run.id);
      refreshCanvas.cancel();
      refreshRunSpace(run);
    },
  });
  return () => {
    if (onUpdate) listeners.delete(onUpdate);
  };
}

export async function restoreResourceRuns() {
  const runs = await invokeCommand<RunObservation[]>("plugin:spaces|list_active_resource_runs");
  for (const run of runs) {
    const title = i18n.global.t(
      run.scope.kind === "application"
        ? "spaces.speech.voice.candidate.generate"
        : "spaces.creation.shared.resourceTask",
    );
    observeResourceRun(run, title);
  }
}

export function cancelResourceRun(runId: string): Promise<void> {
  return requestRunControl("cancel", runId);
}
