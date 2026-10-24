import type { RunObservation } from "../contracts/generated/RunObservation";

export type RunListener = (run: RunObservation) => void;

/**
 * The listeners watching each Run, so one Run is observed once however many places are interested.
 *
 * A Run is observed by exactly one caller — the first one. Everyone after it joins, and each caller
 * gets back the release of its own listener, never of the observation: a view that closes must not
 * stop the Run from being followed for the others.
 */
export function createRunAudience() {
  const listenersByRunId = new Map<string, Set<RunListener>>();
  return {
    /**
     * `listeners` is the audience to feed while the Run is observed; it is `undefined` when the Run
     * is already being observed, and the caller has nothing left to set up.
     */
    join(runId: string, listener?: RunListener) {
      const existing = listenersByRunId.get(runId);
      const listeners = existing ?? new Set<RunListener>();
      if (listener) listeners.add(listener);
      if (!existing) listenersByRunId.set(runId, listeners);
      return {
        listeners: existing ? undefined : (listeners as ReadonlySet<RunListener>),
        release: () => {
          if (listener) listeners.delete(listener);
        },
      };
    },
    leave: (runId: string) => listenersByRunId.delete(runId),
  };
}
