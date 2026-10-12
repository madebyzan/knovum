import { debounce } from "es-toolkit";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { i18n } from "../../../foundation/i18n/instance";
import { invokeCommand } from "../../../foundation/ipc/invoke-command";
import { queryClient } from "../../../foundation/query/query-client";
import { useTaskStore } from "../../../foundation/tasks/task-store";
import type { RunObservation } from "../contracts/generated/RunObservation";
import { useSpaceStore } from "../session/space-store";
import { workflowStateLabelKey } from "../workflows/workflow-state-label";
import { resourcePublicationQueryKey } from "./resource-publication-query";
import { requestRunControl } from "./run-control";
import { loadLatestRunFailure } from "./run-failure";
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

async function reportRunFailure(
  runId: string,
  state: RunObservation["state"],
  stateVersion: number,
  isCurrent: (version: number) => boolean,
) {
  const tasks = useTaskStore();
  try {
    const reason = await loadLatestRunFailure(runId);
    if (!isCurrent(stateVersion)) return;
    tasks.fail(runId, reason ?? i18n.global.t(workflowStateLabelKey(state)));
  } catch {
    if (!isCurrent(stateVersion)) return;
    tasks.fail(runId, i18n.global.t(workflowStateLabelKey(state)));
  }
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
  let latestStateVersion = run.stateVersion;
  observeRun(run.id, {
    update: (next) => {
      latestStateVersion = next.stateVersion;
      for (const listener of listeners) listener(next);
      refreshCanvas();
      if (next.state === "succeeded") tasks.succeed(next.id);
      else if (next.state === "failed" || next.state === "completed_with_errors") {
        void reportRunFailure(
          next.id,
          next.state,
          next.stateVersion,
          (stateVersion) => stateVersion === latestStateVersion,
        );
      } else if (next.state === "cancelled") tasks.cancelled(next.id);
      else tasks.setStatusMessage(next.id, i18n.global.t(workflowStateLabelKey(next.state)));
    },
    error: (cause) => tasks.fail(run.id, getErrorMessage(cause)),
    finished: () => {
      listenersByRunId.delete(run.id);
      refreshCanvas.cancel();
      if (run.scope.kind === "space")
        void queryClient.invalidateQueries({ queryKey: resourcePublicationQueryKey(run.scope.spaceId) });
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
