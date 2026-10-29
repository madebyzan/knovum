import type { ProposedChange } from "../../contracts/generated/ProposedChange";

/**
 * Selection of a proposal's changes that is always closed: choosing a change chooses what it
 * needs, and leaving one out leaves out everything that needs it. Review therefore never offers a
 * subset the backend would shrink on acceptance.
 */
export function toggleChange(
  changes: readonly ProposedChange[],
  selected: ReadonlySet<number>,
  index: number,
  chosen: boolean,
): Set<number> {
  const next = new Set(selected);
  const affected = chosen ? requirementsOf(changes, index) : dependentsOf(changes, index);
  for (const entry of affected) {
    if (chosen) next.add(entry);
    else next.delete(entry);
  }
  return next;
}

export function allChanges(changes: readonly ProposedChange[]): Set<number> {
  return new Set(changes.keys());
}

function requirementsOf(changes: readonly ProposedChange[], index: number): Set<number> {
  return reachable(index, (entry) => changes[entry]?.requires ?? []);
}

function dependentsOf(changes: readonly ProposedChange[], index: number): Set<number> {
  const dependents = new Map<number, number[]>();
  changes.forEach((change, entry) => {
    for (const required of change.requires)
      dependents.set(required, [...(dependents.get(required) ?? []), entry]);
  });
  return reachable(index, (entry) => dependents.get(entry) ?? []);
}

function reachable(start: number, next: (index: number) => readonly number[]): Set<number> {
  const seen = new Set([start]);
  const pending = [start];
  for (let current = pending.pop(); current !== undefined; current = pending.pop()) {
    for (const entry of next(current)) {
      if (seen.has(entry)) continue;
      seen.add(entry);
      pending.push(entry);
    }
  }
  return seen;
}
