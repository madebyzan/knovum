import { onScopeDispose, type Ref, type ShallowRef } from "vue";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { invokeCommand } from "../../../foundation/ipc/invoke-command";
import type { RunObservation } from "../contracts/generated/RunObservation";
import { getConversationTaskError, listActiveConversationRuns } from "./conversation-ipc";
import { observeConversationRun } from "./conversation-run-observation";
import type { ConversationSelectionOptions } from "./conversation-selection";

interface ConversationRunRestorationState {
  spaceId: Ref<string | undefined>;
  selectedConversationId: Ref<string>;
  activeRun: ShallowRef<RunObservation | undefined>;
  activeRunId: ShallowRef<string | undefined>;
  activeConversationId: ShallowRef<string>;
  activeSpaceId: ShallowRef<string | undefined>;
  taskState: ShallowRef<string | undefined>;
  loading: ShallowRef<boolean>;
  error: ShallowRef<string | undefined>;
  activityRevision: ShallowRef<number>;
  refresh: () => Promise<void>;
  select: (conversationId: string, options?: ConversationSelectionOptions) => Promise<void>;
  onDetach: () => void;
}

export function useConversationRunRestoration(state: ConversationRunRestorationState) {
  let observingRunId = "";
  let stopObserving: (() => void) | undefined;
  let revision = 0;
  let disposed = false;

  function detach() {
    revision += 1;
    observingRunId = "";
    stopObserving?.();
    stopObserving = undefined;
    state.onDetach();
  }
  onScopeDispose(() => {
    disposed = true;
    detach();
  });

  function isCurrent(currentRevision: number) {
    return !disposed && revision === currentRevision;
  }

  function isCurrentRun(currentRevision: number, runId: string) {
    return (
      isCurrent(currentRevision) &&
      state.activeRun.value?.id === runId &&
      (!state.activeRunId.value || state.activeRunId.value === runId)
    );
  }

  function canRestore(spaceId: string) {
    return state.spaceId.value === spaceId && !state.loading.value && !state.activeRunId.value;
  }

  async function restoreForSpace(spaceId: string) {
    if (disposed) return;
    if (state.activeSpaceId.value && state.activeSpaceId.value !== spaceId) {
      detach();
      state.activeRun.value = undefined;
      state.activeRunId.value = undefined;
      state.activeConversationId.value = "";
      state.activeSpaceId.value = undefined;
      state.taskState.value = undefined;
      state.loading.value = false;
      state.error.value = undefined;
    }
    if (!canRestore(spaceId)) return;
    const currentRevision = ++revision;
    const [active] = await listActiveConversationRuns(spaceId);
    if (!isCurrent(currentRevision)) return;
    if (!active || !canRestore(spaceId)) return;
    state.activeRunId.value = active.run.id;
    state.activeConversationId.value = active.conversationId;
    state.activeSpaceId.value = spaceId;
    state.taskState.value = active.run.state;
    state.loading.value = true;
    observingRunId = active.run.id;

    const onRun = (run: RunObservation) => {
      if (!isCurrent(currentRevision) || observingRunId !== run.id) return;
      state.activeRun.value = run;
      state.taskState.value = run.state;
      state.activityRevision.value += 1;
      if (run.completedAt !== null) {
        state.activeRunId.value = undefined;
        state.loading.value = false;
        observingRunId = "";
        stopObserving?.();
        stopObserving = undefined;
        if (run.state === "failed" || run.state === "completed_with_errors") {
          void getConversationTaskError(run.id)
            .then((reason) => {
              if (!isCurrentRun(currentRevision, run.id)) return;
              state.error.value = reason ?? `Conversation task ended as ${run.state}`;
              return undefined;
            })
            .catch((cause: unknown) => {
              if (isCurrentRun(currentRevision, run.id)) state.error.value = getErrorMessage(cause);
            });
        }
        void state
          .refresh()
          .then(() => {
            if (!isCurrentRun(currentRevision, run.id)) return;
            if (state.selectedConversationId.value === active.conversationId)
              return state.select(active.conversationId, { branchId: active.branchId });
            return undefined;
          })
          .catch((cause: unknown) => {
            if (isCurrentRun(currentRevision, run.id)) state.error.value = getErrorMessage(cause);
          });
      }
    };
    stopObserving = observeConversationRun({
      run: active.run,
      title: active.title,
      onUpdate: onRun,
      location: { spaceId, conversationId: active.conversationId },
    });
    onRun(active.run);
  }

  async function resume() {
    if (disposed) return;
    const runId = state.activeRunId.value;
    if (!runId || state.taskState.value !== "paused") return;
    const currentRevision = revision;
    state.error.value = undefined;
    try {
      await invokeCommand("plugin:spaces|resume_run", { runId });
    } catch (cause) {
      if (isCurrentRun(currentRevision, runId)) state.error.value = getErrorMessage(cause);
    }
  }

  async function retryModel() {
    if (disposed) return;
    const run = state.activeRun.value;
    if (!run || run.state !== "waiting") return;
    const currentRevision = revision;
    state.error.value = undefined;
    try {
      await invokeCommand("plugin:spaces|retry_builtin_agent_model", {
        runId: run.id,
        expectedVersion: run.stateVersion,
      });
    } catch (cause) {
      if (isCurrentRun(currentRevision, run.id)) state.error.value = getErrorMessage(cause);
    }
  }

  return {
    retryModel,
    restoreForSpace,
    resume,
  };
}
