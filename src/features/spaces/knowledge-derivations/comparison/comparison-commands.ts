import { invokeCommand } from "../../../../foundation/ipc/invoke-command";
import type { DerivationComparisonInput } from "../../contracts/generated/DerivationComparisonInput";

export function isDerivationComparable(nodeId: string) {
  return invokeCommand<boolean>("plugin:spaces|is_derivation_comparable", { nodeId });
}

export function prepareDerivationComparison(nodeId: string) {
  return invokeCommand<DerivationComparisonInput>("plugin:spaces|prepare_derivation_comparison", { nodeId });
}
