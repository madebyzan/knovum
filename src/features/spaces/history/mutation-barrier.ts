import type { SpaceHistoryState } from "./stack";

const idleWaiters = new WeakMap<SpaceHistoryState, Set<() => void>>();
const closedAdmissions = new WeakSet<SpaceHistoryState>();

export function beginSpaceMutation(history: SpaceHistoryState) {
  if (closedAdmissions.has(history)) throw new Error("Space session is transitioning");
  history.pendingMutations += 1;
  let active = true;
  return () => {
    if (!active) return;
    active = false;
    endSpaceMutation(history);
  };
}

function endSpaceMutation(history: SpaceHistoryState) {
  history.pendingMutations -= 1;
  if (history.pendingMutations !== 0) return;
  const waiters = idleWaiters.get(history);
  if (!waiters) return;
  idleWaiters.delete(history);
  for (const resolve of waiters) resolve();
}

export async function waitForSpaceMutations(history: SpaceHistoryState) {
  if (history.pendingMutations === 0) return;
  await new Promise<void>((resolve) => {
    const waiters = idleWaiters.get(history) ?? new Set();
    waiters.add(resolve);
    idleWaiters.set(history, waiters);
  });
}

export function closeSpaceMutationAdmission(history: SpaceHistoryState) {
  closedAdmissions.add(history);
  return () => closedAdmissions.delete(history);
}
