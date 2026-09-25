import type { SpaceState } from "../session/session-state";

export function createWorkflowPanelActions(state: SpaceState) {
  function openWorkflowCapture(nodeIds: string[]) {
    state.workflowCaptureSelection.value = [...nodeIds];
    state.workflowMode.value = "capture";
  }

  function openWorkflowUse(nodeIds: string[]) {
    state.workflowCaptureSelection.value = [...nodeIds];
    state.workflowMode.value = "use";
  }

  return { openWorkflowCapture, openWorkflowUse };
}
