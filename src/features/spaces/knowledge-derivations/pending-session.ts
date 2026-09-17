import { shallowReactive } from "vue";

export interface PendingDerivation {
  outputNodeId: string;
  sourceNodeIds: readonly string[];
}

const entries = shallowReactive(new Map<string, PendingDerivation>());

export function beginPendingDerivation(outputNodeId: string, sourceNodeIds: readonly string[]) {
  entries.set(outputNodeId, { outputNodeId, sourceNodeIds: [...sourceNodeIds] });
}

export function endPendingDerivation(outputNodeId: string) {
  entries.delete(outputNodeId);
}

export function pendingDerivations() {
  return [...entries.values()];
}
