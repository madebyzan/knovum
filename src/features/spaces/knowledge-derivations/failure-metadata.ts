export type { DerivationFailure } from "../contracts/generated/DerivationFailure";
import { invokeCommand as invoke } from "../../../foundation/ipc/invoke-command";
import type { DerivationFailure } from "../contracts/generated/DerivationFailure";

export function loadDerivationFailure(nodeId: string) {
  return invoke<DerivationFailure | null>("plugin:spaces|get_derivation_failure", { nodeId });
}
