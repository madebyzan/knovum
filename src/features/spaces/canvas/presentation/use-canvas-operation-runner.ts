import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import { reportEditorProtection } from "../../workspace/editor-protection-feedback";
import { useNodeOperationGate, type NodeOperationEffect } from "../use-node-operation-gate";

export function useCanvasOperationRunner() {
  const gate = useNodeOperationGate();

  function run(effect: NodeOperationEffect, operation: () => Promise<unknown>) {
    if (!gate.allows(effect)) return;
    void Promise.resolve()
      .then(() => (gate.allows(effect) ? operation() : undefined))
      .catch((error: unknown) => {
        if (!reportEditorProtection(error)) toast.error(getErrorMessage(error));
      });
  }

  return { allows: gate.allows, run };
}
