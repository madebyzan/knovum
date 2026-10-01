import { toast } from "vue-sonner";

import { errorMessage } from "../../../../foundation/errors/error-format";
import { useNodeOperationGate, type NodeOperationEffect } from "../../canvas/node-operation-gate";

export function useCanvasOperationRunner() {
  const gate = useNodeOperationGate();

  function run(effect: NodeOperationEffect, operation: () => Promise<unknown>) {
    if (!gate.allows(effect)) return;
    void Promise.resolve()
      .then(() => (gate.allows(effect) ? operation() : undefined))
      .catch((error) => toast.error(errorMessage(error)));
  }

  return { allows: gate.allows, run };
}
