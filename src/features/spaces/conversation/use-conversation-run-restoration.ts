import { shallowRef, type Ref, type ShallowRef } from "vue";

import { invokeCommand as invoke } from "../../../foundation/ipc/invoke-command";
import { decideAgentPermission } from "../agent-creation/agent-artifacts";
import type { ApprovalSummary } from "../contracts/generated/ActionApproval";
import type { AssistantTaskUpdate } from "../contracts/generated/AssistantTaskUpdate";
import type { RunSummary } from "../contracts/generated/RunSummary";
import type { WaitReason } from "../contracts/generated/WaitReason";
import {
  decideAssistantApproval,
  getAssistantTaskError,
  listActiveAssistantRuns,
  listAssistantApprovals,
  pendingAssistantPermission,
} from "./conversation-ipc";
import { observeConversationRun } from "./conversation-run-observation";

type Permission = Extract<AssistantTaskUpdate, { type: "permission_requested" }>;
const stopNoObservation = () => {};

export function useConversationRunRestoration(state: {
  spaceId: Ref<string | undefined>;
  selectedConversationId: Ref<string>;
  activeRunId: ShallowRef<string | undefined>;
  activeConversationId: ShallowRef<string>;
  activeSpaceId: ShallowRef<string | undefined>;
  taskState: ShallowRef<string | undefined>;
  waitReason: ShallowRef<WaitReason | null>;
  loading: ShallowRef<boolean>;
  permission: ShallowRef<Permission | undefined>;
  error: ShallowRef<string | undefined>;
  refresh: () => Promise<void>;
  select: (id: string, skipRemember?: boolean, branchId?: string) => Promise<void>;
  onDetach: () => void;
}) {
  let observingRunId = "";
  let stopObserving = stopNoObservation;
  const decidingPermission = shallowRef(false);
  const approvals = shallowRef<ApprovalSummary[]>([]);
  const decidingApproval = shallowRef(false);

  async function refreshPending(runId: string, spaceId: string) {
    const [pending, actionApprovals] = await Promise.all([
      pendingAssistantPermission(runId),
      listAssistantApprovals(spaceId, runId),
    ]);
    if (state.activeRunId.value !== runId) return;
    state.permission.value = pending ?? undefined;
    approvals.value = actionApprovals;
  }

  async function restoreForSpace(spaceId: string) {
    if (state.activeSpaceId.value && state.activeSpaceId.value !== spaceId) {
      state.onDetach();
      stopObserving();
      observingRunId = "";
      state.activeRunId.value = undefined;
      state.activeConversationId.value = "";
      state.activeSpaceId.value = undefined;
      state.taskState.value = undefined;
      state.waitReason.value = null;
      state.loading.value = false;
      state.error.value = undefined;
      state.permission.value = undefined;
      approvals.value = [];
    }
    if (state.loading.value || state.activeRunId.value) return;
    const [active] = await listActiveAssistantRuns(spaceId);
    if (!active || state.spaceId.value !== spaceId || state.loading.value || state.activeRunId.value) return;
    state.activeRunId.value = active.run.id;
    state.activeConversationId.value = active.conversationId;
    state.activeSpaceId.value = spaceId;
    state.taskState.value = active.run.state;
    state.waitReason.value = active.run.waitReason;
    state.loading.value = true;
    observingRunId = active.run.id;

    const onRun = (run: RunSummary) => {
      if (observingRunId !== run.id) return;
      state.taskState.value = run.state;
      state.waitReason.value = run.waitReason;
      if (run.completedAt !== null) {
        state.permission.value = undefined;
        approvals.value = [];
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
      } else if (run.state === "waiting") {
        void refreshPending(run.id, spaceId).catch((cause: unknown) => {
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
      await invoke("plugin:spaces|resume_run", { runId: state.activeRunId.value });
    } catch (cause) {
      state.error.value = String(cause);
    }
  }

  async function decidePermission(optionId: string | null) {
    const pending = state.permission.value;
    if (!pending || decidingPermission.value) return;
    decidingPermission.value = true;
    state.error.value = undefined;
    try {
      await decideAgentPermission(pending.session_id, pending.request_id, optionId);
      if (state.permission.value?.request_id === pending.request_id) state.permission.value = undefined;
    } catch (cause) {
      state.error.value = String(cause);
    } finally {
      decidingPermission.value = false;
    }
  }

  async function decideApproval(approvalId: string, approved: boolean) {
    if (decidingApproval.value || !approvals.value.some(({ id }) => id === approvalId)) return;
    decidingApproval.value = true;
    state.error.value = undefined;
    try {
      await decideAssistantApproval(approvalId, approved);
      approvals.value = approvals.value.filter(({ id }) => id !== approvalId);
    } catch (cause) {
      state.error.value = String(cause);
    } finally {
      decidingApproval.value = false;
    }
  }

  return {
    restoreForSpace,
    resume,
    decidingPermission,
    decidePermission,
    approvals,
    decidingApproval,
    decideApproval,
    refreshPending,
  };
}
