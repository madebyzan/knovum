import { type Ref, type ShallowRef } from "vue";

import { invokeCommand } from "../../../foundation/ipc/invoke-command";
import type { RunObservation } from "../contracts/generated/RunObservation";
import { getAssistantTaskError, listActiveAssistantRuns } from "./conversation-ipc";
import { observeConversationRun } from "./conversation-run-observation";

const stopNoObservation = () => {};

export function useConversationRunRestoration(state: {
  spaceId: Ref<string | undefined>;
  selectedConversationId: Ref<string>;
  activeRunId: ShallowRef<string | undefined>;
  activeConversationId: ShallowRef<string>;
  activeSpaceId: ShallowRef<string | undefined>;
  taskState: ShallowRef<string | undefined>;
  loading: ShallowRef<boolean>;
  error: ShallowRef<string | undefined>;
  activityRevision: ShallowRef<number>;
  refresh: () => Promise<void>;
  select: (id: string, skipRemember?: boolean, branchId?: string) => Promise<void>;
  onDetach: () => void;
}) {
  let observingRunId = "";
  let stopObserving = stopNoObservation;
  async function restoreForSpace(spaceId: string) {
    if (state.activeSpaceId.value && state.activeSpaceId.value !== spaceId) {
      state.onDetach();
      stopObserving();
      observingRunId = "";
      state.activeRunId.value = undefined;
      state.activeConversationId.value = "";
      state.activeSpaceId.value = undefined;
      state.taskState.value = undefined;
      state.loading.value = false;
      state.error.value = undefined;
    }
    if (state.loading.value || state.activeRunId.value) return;
    const [active] = await listActiveAssistantRuns(spaceId);
    if (!active || state.spaceId.value !== spaceId || state.loading.value || state.activeRunId.value) return;
    state.activeRunId.value = active.run.id;
    state.activeConversationId.value = active.conversationId;
    state.activeSpaceId.value = spaceId;
    state.taskState.value = active.run.state;
    state.loading.value = true;
    observingRunId = active.run.id;

    const onRun = (run: RunObservation) => {
      if (observingRunId !== run.id) return;
      state.taskState.value = run.state;
      state.activityRevision.value += 1;
      if (run.completedAt !== null) {
        state.activeRunId.value = undefined;
        state.loading.value = false;
        observingRunId = "";
        stopObserving();
        if (run.state === "failed" || run.state === "completed_with_errors") {
          void getAssistantTaskError(run.id)
            .then((reason) => {
              state.error.value = reason ?? `Assistant task ended as ${run.state}`;
              return undefined;
            })
            .catch((cause: unknown) => {
              state.error.value = String(cause);
            });
        }
        void state
          .refresh()
          .then(() => {
            if (state.selectedConversationId.value === active.conversationId)
              return state.select(active.conversationId, false, active.branchId);
            return undefined;
          })
          .catch((cause: unknown) => {
            state.error.value = String(cause);
          });
      }
    };
    stopObserving = observeConversationRun(active.run, active.title, onRun, {
      spaceId,
      conversationId: active.conversationId,
    });
    onRun(active.run);
  }

  async function resume() {
    if (!state.activeRunId.value || state.taskState.value !== "paused") return;
    state.error.value = undefined;
    try {
      await invokeCommand("plugin:spaces|resume_run", { runId: state.activeRunId.value });
    } catch (cause) {
      state.error.value = String(cause);
    }
  }

  return {
    restoreForSpace,
    resume,
  };
}
