import { onScopeDispose, readonly, shallowRef, watch, type DeepReadonly } from "vue";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { invokeCommand } from "../../../foundation/ipc/invoke-command";
import type { RunObservation } from "../contracts/generated/RunObservation";
import type { SpeechProjectSnapshot } from "../contracts/generated/SpeechProjectSnapshot";
import { observeResourceRun } from "../runs/resource-run-observation";
import { useSpaceStore } from "../session/space-store";
export interface SpeechRunOwner {
  project: () => DeepReadonly<SpeechProjectSnapshot> | undefined;
  reloadProject: (projectId: string) => Promise<void>;
}

const TERMINAL_STATES = new Set(["succeeded", "completed_with_errors", "failed", "cancelled"]);

export function useSpeechRunObservation(owner: SpeechRunOwner) {
  const store = useSpaceStore();
  const runState = shallowRef<RunObservation["state"]>();
  const error = shallowRef<string>();
  let stopObservation: (() => unknown) | undefined;
  let observationEpoch = 0;
  function stopRunObservation() {
    observationEpoch += 1;
    stopObservation?.();
    stopObservation = undefined;
    runState.value = undefined;
    error.value = undefined;
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
        void owner.reloadProject(projectId).catch((cause) => {
          reportObservationError(cause, epoch, projectId, spaceId);
        });
        void store.refreshActiveSpace().catch((cause) => {
          reportObservationError(cause, epoch, projectId, spaceId);
        });
      }
    });
  }

  function reportObservationError(cause: unknown, epoch: number, projectId: string, spaceId: string) {
    if (
      epoch === observationEpoch &&
      store.activeSpaceId === spaceId &&
      owner.project()?.project.id === projectId
    )
      error.value = getErrorMessage(cause);
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
          error.value = getErrorMessage(cause);
        }
      }
    },
    { immediate: true },
  );

  onScopeDispose(stopRunObservation);
  return { runState: readonly(runState), error: readonly(error), attachRun, reset: stopRunObservation };
}
