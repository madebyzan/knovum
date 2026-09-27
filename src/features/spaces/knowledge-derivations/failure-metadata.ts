export type { DerivationFailure } from "../contracts/generated/DerivationFailure";
import { invokeCommand } from "../../../foundation/ipc/invoke-command";
import type { DerivationFailure } from "../contracts/generated/DerivationFailure";

export function loadDerivationFailure(nodeId: string) {
  return invokeCommand<DerivationFailure | null>("plugin:spaces|get_derivation_failure", { nodeId });
}
