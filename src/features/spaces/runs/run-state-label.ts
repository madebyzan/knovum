/** Localizes a Run lifecycle state for the feature surfaces that observe their own Runs. */
export function runStateLabelKey(state: string) {
  return `spaces.runs.states.${state === "completed_with_errors" ? "completedWithErrors" : state}`;
}
