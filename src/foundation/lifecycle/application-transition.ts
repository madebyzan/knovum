let transitioning = false;

export function isApplicationTransitioning(): boolean {
  return transitioning;
}

// Quit, hide-after-save and update installation share one admission boundary.
// Registered saves still run normally inside the transition.
export async function runApplicationTransition(operation: () => Promise<void>): Promise<void> {
  if (transitioning) throw new Error("An application transition is already running.");
  transitioning = true;
  try {
    await operation();
  } finally {
    transitioning = false;
  }
}
