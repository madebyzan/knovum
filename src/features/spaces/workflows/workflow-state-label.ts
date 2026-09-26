export function workflowStateLabelKey(state: string) {
  return `spaces.creation.workflowStates.${state === "completed_with_errors" ? "completedWithErrors" : state}`;
}
