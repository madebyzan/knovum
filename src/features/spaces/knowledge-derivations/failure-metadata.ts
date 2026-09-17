import { invoke } from "@tauri-apps/api/core";

export interface DerivationFailure {
  actionKind: string;
  error: string;
  completedAt: number;
}

export function loadDerivationFailure(nodeId: string) {
  return invoke<DerivationFailure | null>("plugin:spaces|get_derivation_failure", { nodeId });
}
