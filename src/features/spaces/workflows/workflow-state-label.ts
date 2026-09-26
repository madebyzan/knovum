export function workflowStateLabelKey(state: string) {
  return `spaces.creation.workflow.states.${state === "completed_with_errors" ? "completedWithErrors" : state}`;
}
