/** One executor a capability can run on, as the chooser presents it. */
export interface ModelChoiceOption {
  readonly id: string;
  readonly name: string;
  readonly detail?: string | undefined;
  readonly runnable: boolean;
  readonly preferred: boolean;
}

export type ModelChoiceDecision =
  | { readonly kind: "run"; readonly id: string }
  | { readonly kind: "choose"; readonly options: readonly ModelChoiceOption[]; readonly selectedId: string }
  | { readonly kind: "none" };

/** The Settings preference when it can run, otherwise the first executor that can. */
export function preferredChoiceId(options: readonly ModelChoiceOption[]): string | undefined {
  return (options.find((option) => option.preferred) ?? options[0])?.id;
}

/**
 * One runnable executor runs without asking; several ask, with the Settings preference preselected.
 * Options that cannot run yet are not offered — preparing them belongs to the readiness path.
 */
export function decideModelChoice(options: readonly ModelChoiceOption[]): ModelChoiceDecision {
  const runnable = options.filter((option) => option.runnable);
  const selectedId = preferredChoiceId(runnable);
  if (!selectedId) return { kind: "none" };
  if (runnable.length === 1) return { kind: "run", id: selectedId };
  return { kind: "choose", options: runnable, selectedId };
}
