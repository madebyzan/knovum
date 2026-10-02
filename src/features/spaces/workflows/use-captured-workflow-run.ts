import { onBeforeUnmount, ref, shallowRef } from "vue";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import type { RunObservation } from "../contracts/generated/RunObservation";
import type { RunState } from "../contracts/generated/RunState";
import type { SavedCapturedWorkflow } from "../contracts/generated/SavedCapturedWorkflow";
import { cancelResourceRun, observeResourceRun } from "../runs/resource-run-observation";
import { useSpaceStore } from "../session/space-store";
import {
  listCapturedRunOutputs,
  retryCapturedWorkflowRun,
  startCapturedWorkflow,
} from "./captured-workflow-ipc";

interface CapturedWorkflowRunInput {
  workflow: SavedCapturedWorkflow;
  spaceId: string;
  bindings: Record<string, string[]>;
  requestId: string;
  isCurrent: () => boolean;
  onStarted: (run: RunObservation) => void;
}

export function useCapturedWorkflowRun() {
  const store = useSpaceStore();
  const runId = ref("");
  const runState = ref<RunState | "">("");
  const cancelling = ref(false);
  const submitting = ref(false);
  const retrying = ref(false);
  const error = ref("");
  const outputNodes = shallowRef<{ outputSlotId: string; name: string; nodeId: string }[]>([]);
  let stopObservation: (() => void) | undefined;
  let disposed = false;

  function reset() {
    stopObservation?.();
    stopObservation = undefined;
    runId.value = "";
    runState.value = "";
    cancelling.value = false;
    outputNodes.value = [];
    error.value = "";
  }

  async function refreshOutputs(id: string) {
    try {
      const outputs = await listCapturedRunOutputs(id);
      if (runId.value === id && !disposed) outputNodes.value = outputs;
    } catch (cause) {
      if (runId.value === id && !disposed) error.value = getErrorMessage(cause);
    }
  }

  function attach(run: RunObservation, title: string, active: boolean) {
    stopObservation?.();
    if (active && !disposed) {
      runId.value = run.id;
      runState.value = run.state;
      if (run.completedAt !== null) void refreshOutputs(run.id);
    }
    if (run.completedAt !== null) return;
    const stop = observeResourceRun(run, title, (next) => {
      if (next.id !== runId.value || disposed) return;
      runState.value = next.state;
      if (next.completedAt !== null) {
        cancelling.value = false;
        void refreshOutputs(next.id);
      }
    });
    if (active && !disposed) stopObservation = stop;
    else stop();
  }

  async function start(input: CapturedWorkflowRunInput) {
    if (submitting.value) return;
    submitting.value = true;
    error.value = "";
    try {
      const started = await startCapturedWorkflow({
        workflowId: input.workflow.workflowId,
        expectedVersion: input.workflow.version,
        requestId: input.requestId,
        spaceId: input.spaceId,
        bindings: Object.fromEntries(Object.entries(input.bindings).map(([id, nodes]) => [id, [...nodes]])),
      });
      input.onStarted(started);
      attach(started, input.workflow.name, !disposed && input.isCurrent());
    } catch (cause) {
      if (!disposed && input.isCurrent()) error.value = getErrorMessage(cause);
    } finally {
      submitting.value = false;
    }
  }

  async function cancel() {
    if (!runId.value || cancelling.value) return;
    cancelling.value = true;
    try {
      await cancelResourceRun(runId.value);
    } catch (cause) {
      cancelling.value = false;
      error.value = getErrorMessage(cause);
    }
  }

  async function retry(title: string, onStarted: (run: RunObservation) => void) {
    if (!runId.value || retrying.value) return;
    const sourceRunId = runId.value;
    retrying.value = true;
    error.value = "";
    try {
      const started = await retryCapturedWorkflowRun(sourceRunId);
      onStarted(started);
      attach(started, title, !disposed && runId.value === sourceRunId);
    } catch (cause) {
      error.value = getErrorMessage(cause);
    } finally {
      retrying.value = false;
    }
  }

  async function locate(nodeId: string) {
    try {
      await store.refreshActiveSpace();
      store.requestReveal([nodeId], "focus");
    } catch (cause) {
      error.value = getErrorMessage(cause);
    }
  }

  onBeforeUnmount(() => {
    disposed = true;
    stopObservation?.();
  });

  return {
    runId,
    runState,
    cancelling,
    submitting,
    retrying,
    error,
    outputNodes,
    reset,
    attach,
    start,
    cancel,
    retry,
    locate,
  };
}
