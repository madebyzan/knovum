import { shallowRef, type Ref } from "vue";

import {
  preflightWorkflow,
  publishWorkflow,
  startWorkflow,
  startWorkflowStepTest,
  validateWorkflow,
  type WorkflowDraft,
} from "./workflow-client";

type Perform = (action: () => Promise<void>) => Promise<void>;

export function useWorkflowExecution(draft: Ref<WorkflowDraft | undefined>, perform: Perform) {
  const diagnostics = shallowRef<string[]>([]);
  const sampleInput = shallowRef("{}");
  const breakpointNodeIds = shallowRef<string[]>([]);
  const selectedTestNodeId = shallowRef("");

  async function validate() {
    if (!draft.value) return;
    diagnostics.value = [];
    try {
      await validateWorkflow(draft.value.definition);
      const readiness = await preflightWorkflow(draft.value.definition);
      diagnostics.value = readiness.map(({ nodeId, message }) => `${nodeId}: ${message}`);
    } catch (reason) {
      diagnostics.value = Array.isArray(reason)
        ? reason.map((item) => String((item as { message?: unknown }).message ?? item))
        : [String(reason)];
    }
  }

  async function publish() {
    if (!draft.value) return;
    await perform(async () => {
      await publishWorkflow(draft.value!.id, draft.value!.revision);
    });
  }

  async function run(testMode = false) {
    if (!draft.value) return;
    await perform(async () => {
      const readiness = await preflightWorkflow(draft.value!.definition);
      if (readiness.length) {
        throw new Error(readiness.map(({ nodeId, message }) => `${nodeId}: ${message}`).join("\n"));
      }
      const version = await publishWorkflow(draft.value!.id, draft.value!.revision);
      await startWorkflow({
        spaceId: draft.value!.spaceId,
        workflowId: draft.value!.id,
        version: version.version,
        input: JSON.parse(sampleInput.value) as unknown,
        testMode,
        breakpointNodeIds: breakpointNodeIds.value,
      });
    });
  }

  async function testStep() {
    if (!draft.value || !selectedTestNodeId.value) return;
    await perform(async () => {
      const readiness = await preflightWorkflow(draft.value!.definition);
      const issue = readiness.find(({ nodeId }) => nodeId === selectedTestNodeId.value);
      if (issue) throw new Error(`${issue.nodeId}: ${issue.message}`);
      const version = await publishWorkflow(draft.value!.id, draft.value!.revision);
      await startWorkflowStepTest({
        spaceId: draft.value!.spaceId,
        workflowId: draft.value!.id,
        version: version.version,
        nodeId: selectedTestNodeId.value,
        input: JSON.parse(sampleInput.value) as unknown,
      });
    });
  }

  function reset(value: WorkflowDraft) {
    breakpointNodeIds.value = [];
    selectedTestNodeId.value = value.definition.nodes[0]?.id ?? "";
  }

  function toggleBreakpoint(nodeId: string) {
    breakpointNodeIds.value = breakpointNodeIds.value.includes(nodeId)
      ? breakpointNodeIds.value.filter((candidate) => candidate !== nodeId)
      : [...breakpointNodeIds.value, nodeId];
  }

  return {
    diagnostics,
    sampleInput,
    breakpointNodeIds,
    selectedTestNodeId,
    validate,
    publish,
    run,
    test: () => run(true),
    testStep,
    reset,
    toggleBreakpoint,
  };
}
