interface SpaceMutationState {
  pendingMutations: number;
}

const idleWaitersByHistory = new WeakMap<SpaceMutationState, Set<() => void>>();
const closedAdmissions = new WeakSet<SpaceMutationState>();

export function beginSpaceMutation(history: SpaceMutationState) {
  if (closedAdmissions.has(history)) throw new Error("Space session is transitioning");
  history.pendingMutations += 1;
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

export function closeSpaceMutationAdmission(history: SpaceMutationState) {
  closedAdmissions.add(history);
  return () => closedAdmissions.delete(history);
}
