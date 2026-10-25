/**
 * The canvas node a Run publishes its single output into. The Action engine reserves it under this
 * identity as the Run begins (`derivation/action_request/output`), so a retry finds it again and the
 * canvas can tell when the work it asked for has taken its place.
 */
export function runOutputNodeId(runId: string) {
  return `${runId}-output`;
}
