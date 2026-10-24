import { debounce } from "es-toolkit";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { i18n } from "../../../foundation/i18n/instance";
import { invokeCommand } from "../../../foundation/ipc/invoke-command";
import { queryClient } from "../../../foundation/query/query-client";
import { useTaskStore } from "../../../foundation/tasks/task-store";
import type { RunObservation } from "../contracts/generated/RunObservation";
import { runStateLabelKey } from "../runs/run-state-label";
import { useSpaceStore } from "../session/space-store";
import { resourcePublicationQueryKey } from "./resource-publication-query";
import { createRunAudience, type RunListener } from "./run-audience";
import { requestRunControl } from "./run-control";
import { loadLatestRunFailure } from "./run-failure";
import { observeRun } from "./run-observation";

const audience = createRunAudience();
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
    tasks.fail(runId, reason ?? i18n.global.t(runStateLabelKey(state)));
  } catch {
    if (!isCurrent(stateVersion)) return;
    tasks.fail(runId, i18n.global.t(runStateLabelKey(state)));
  }
}

export function observeResourceRun(run: RunObservation, title: string, onUpdate?: RunListener) {
  const { listeners, release } = audience.join(run.id, onUpdate);
  if (!listeners) return release;
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
      else tasks.setStatusMessage(next.id, i18n.global.t(runStateLabelKey(next.state)));
    },
    error: (cause) => tasks.fail(run.id, getErrorMessage(cause)),
    finished: () => {
      audience.leave(run.id);
      refreshCanvas.cancel();
      if (run.scope.kind === "space")
        void queryClient.invalidateQueries({ queryKey: resourcePublicationQueryKey(run.scope.spaceId) });
      refreshRunSpace(run);
    },
  });
  return release;
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
