interface SpaceMutationState {
  pendingMutations: number;
}

const idleWaitersByHistory = new WeakMap<SpaceMutationState, Set<() => void>>();
const closedAdmissions = new WeakSet<SpaceMutationState>();
const admissionsByHistory = new WeakMap<SpaceMutationState, number>();

export function beginSpaceMutation(history: SpaceMutationState) {
  if (closedAdmissions.has(history)) throw new Error("Space session is transitioning");
  history.pendingMutations += 1;
  admissionsByHistory.set(history, spaceMutationAdmissions(history) + 1);
  let active = true;
  return () => {
    if (!active) return;
    active = false;
    endSpaceMutation(history);
  };
}

function endSpaceMutation(history: SpaceMutationState) {
  history.pendingMutations -= 1;
  if (history.pendingMutations !== 0) return;
  const waiters = idleWaitersByHistory.get(history);
  if (!waiters) return;
  idleWaitersByHistory.delete(history);
  for (const resolve of waiters) resolve();
}

export async function waitForSpaceMutations(history: SpaceMutationState) {
  if (history.pendingMutations === 0) return;
  await new Promise<void>((resolve) => {
    const waiters = idleWaitersByHistory.get(history) ?? new Set();
    waiters.add(resolve);
    idleWaitersByHistory.set(history, waiters);
  });
}

/**
 * Counts every mutation the session has admitted. A read that sees the same count before and after
 * it ran overlapped no local write, so it cannot be older than what the session shows.
 */
export function spaceMutationAdmissions(history: SpaceMutationState) {
  return admissionsByHistory.get(history) ?? 0;
}

export function closeSpaceMutationAdmission(history: SpaceMutationState) {
  closedAdmissions.add(history);
  return () => closedAdmissions.delete(history);
}
