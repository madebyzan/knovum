import { onScopeDispose, readonly, shallowRef, watch } from "vue";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { invokeCommand } from "../../../foundation/ipc/invoke-command";
import type { RunObservation } from "../contracts/generated/RunObservation";
import type { SpeechProjectSnapshot } from "../contracts/generated/SpeechProjectSnapshot";
import { observeResourceRun } from "../runs/resource-run-observation";
import { useSpaceStore } from "../session/space-store";
export interface SpeechRunOwner {
  project: () => SpeechProjectSnapshot | undefined;
  reloadProject: (projectId: string) => Promise<void>;
  setError: (message?: string) => void;
}

const TERMINAL_STATES = new Set(["succeeded", "completed_with_errors", "failed", "cancelled"]);

export function useSpeechRunObservation(owner: SpeechRunOwner) {
  const store = useSpaceStore();
  const runState = shallowRef<RunObservation["state"]>();
  let stopObservation: (() => unknown) | undefined;
  let observationEpoch = 0;
  function stopRunObservation() {
    observationEpoch += 1;
    stopObservation?.();
    stopObservation = undefined;
    runState.value = undefined;
  }

  function attachRun(run: RunObservation, projectId: string, spaceId: string) {
    stopRunObservation();
    const epoch = observationEpoch;
    runState.value = run.state;
    stopObservation = observeResourceRun(run, owner.project()?.project.title ?? "Speech", (next) => {
      if (
        epoch !== observationEpoch ||
        store.activeSpaceId !== spaceId ||
        owner.project()?.project.id !== projectId
      )
        return;
      runState.value = next.state;
      if (TERMINAL_STATES.has(next.state)) {
        void owner.reloadProject(projectId).catch((cause) => owner.setError(getErrorMessage(cause)));
        void store.refreshActiveSpace().catch((cause) => owner.setError(getErrorMessage(cause)));
      }
    });
  }

  function isCurrentObservation(epoch: number, runId: string, spaceId: string) {
    return (
      epoch === observationEpoch &&
      owner.project()?.project.activeRunId === runId &&
      store.activeSpaceId === spaceId
    );
  }

  watch(
    () => owner.project()?.project.activeRunId,
    async (runId) => {
      stopRunObservation();
      const epoch = observationEpoch;
      const projectId = owner.project()?.project.id;
      const spaceId = store.activeSpaceId;
      if (!runId || !projectId || !spaceId) return;
      try {
        const run = await invokeCommand<RunObservation>("plugin:spaces|get_run", { runId });
        if (isCurrentObservation(epoch, runId, spaceId)) attachRun(run, projectId, spaceId);
      } catch (cause) {
        if (isCurrentObservation(epoch, runId, spaceId)) {
          owner.setError(getErrorMessage(cause));
        }
      }
    },
    { immediate: true },
  );

  onScopeDispose(stopRunObservation);
  return { runState: readonly(runState), attachRun, reset: stopRunObservation };
}
