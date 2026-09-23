import { save } from "@tauri-apps/plugin-dialog";
import { writeTextFile } from "@tauri-apps/plugin-fs";
import { storeToRefs } from "pinia";
import { computed, onMounted, onUnmounted, shallowRef, watch } from "vue";

import { useSpaceStore } from "../session/space-store";
import {
  continueRunStep,
  controlRun,
  decideActionApproval,
  decideRunStepReview,
  getRunDiagnostics,
  listPendingActionApprovals,
  listRunBatchItems,
  listRunEvents,
  listRuns,
  listRunSteps,
  reconcileRunStep,
  retryRun,
  submitRunStepInput,
  type ActionApproval,
  type RunBatchItem,
  type RunEvent,
  type RunStep,
  type RunSummary,
} from "./run-client";

export function useRunCenter() {
  const { activeSpaceId } = storeToRefs(useSpaceStore());
  const runs = shallowRef<RunSummary[]>([]);
  const steps = shallowRef<RunStep[]>([]);
  const batchItems = shallowRef<RunBatchItem[]>([]);
  const events = shallowRef<RunEvent[]>([]);
  const approvals = shallowRef<ActionApproval[]>([]);
  const selectedRunId = shallowRef("");
  const selectedStepIds = shallowRef<string[]>([]);
  const selectedRun = computed(() => runs.value.find(({ id }) => id === selectedRunId.value));
  let refreshTimer: number | undefined;

  async function refresh() {
    if (!activeSpaceId.value) return;
    runs.value = await listRuns(activeSpaceId.value);
    approvals.value = await listPendingActionApprovals(activeSpaceId.value);
    if (!selectedRunId.value && runs.value[0]) await select(runs.value[0].id);
  }
  async function refreshDetails(id = selectedRunId.value) {
    if (!id) return;
    [steps.value, batchItems.value, events.value] = await Promise.all([
      listRunSteps(id),
      listRunBatchItems(id),
      listRunEvents(id),
    ]);
  }
  async function select(id: string) {
    selectedRunId.value = id;
    await refreshDetails(id);
    selectedStepIds.value = [];
  }
  async function decideApproval(approval: ActionApproval, approved: boolean) {
    await decideActionApproval(approval.id, approved);
    await refresh();
  }
  async function submitInput(step: RunStep, value: unknown) {
    await submitRunStepInput(step.id, step.stateVersion, value);
    await select(step.runId);
  }
  async function review(step: RunStep, approved: boolean) {
    await decideRunStepReview(step.id, step.stateVersion, approved);
    await select(step.runId);
  }
  async function reconcile(step: RunStep, decision: "retry" | "fail") {
    await reconcileRunStep(step.id, step.stateVersion, decision);
    await refresh();
    await select(step.runId);
  }
  async function continueStep(step: RunStep) {
    await continueRunStep(step.id, step.stateVersion);
    await refresh();
    await select(step.runId);
  }
  async function control(action: "cancel" | "pause" | "resume", id: string) {
    await controlRun(action, id);
    await refresh();
    if (selectedRunId.value) steps.value = await listRunSteps(selectedRunId.value);
  }
  async function retry(id: string) {
    const retried = await retryRun(id, selectedRunId.value === id ? selectedStepIds.value : []);
    await refresh();
    await select(retried.id);
  }
  function toggleStep(stepId: string) {
    selectedStepIds.value = selectedStepIds.value.includes(stepId)
      ? selectedStepIds.value.filter((id) => id !== stepId)
      : [...selectedStepIds.value, stepId];
  }

  watch(activeSpaceId, refresh);
  onMounted(() => {
    void refresh();
    refreshTimer = globalThis.setInterval(() => {
      void refresh();
      void refreshDetails();
    }, 1_000);
  });
  onUnmounted(() => globalThis.clearInterval(refreshTimer));
  return {
    approvals,
    batchItems,
    continueStep,
    control,
    decideApproval,
    events,
    exportDiagnostics,
    reconcile,
    refreshDetails,
    retry,
    review,
    runs,
    select,
    selectedRun,
    selectedStepIds,
    steps,
    submitInput,
    toggleStep,
  };
}

async function exportDiagnostics(runId: string) {
  const destination = await save({ defaultPath: `knovum-run-${runId.slice(0, 8)}-diagnostics.json` });
  if (!destination) return;
  const bundle = await getRunDiagnostics(runId);
  await writeTextFile(destination, `${JSON.stringify(bundle, null, 2)}\n`);
}
