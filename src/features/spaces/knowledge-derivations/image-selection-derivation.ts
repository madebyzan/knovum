import type { ImageExecutionTarget } from "../contracts/generated/ImageExecutionTarget";
import type { ImageSelection } from "../contracts/generated/ImageSelection";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import { defaultImageTarget, deriveImage } from "./image-operation";
import type { DerivationActionStore } from "./start-action";

interface ImageSelectionDerivationInput {
  selection: ImageSelection;
  execution: Extract<ImageExecutionTarget, { type: "local" }>;
  effect: "extract" | "repair";
}

export async function deriveImageSelection(
  store: DerivationActionStore,
  source: SpaceCanvasNode,
  { selection, execution, effect }: ImageSelectionDerivationInput,
) {
  const operation =
    effect === "extract"
      ? { kind: "extract-selection" as const, selection }
      : {
          kind: "remove-selected-object" as const,
          selection,
          selectionModel: execution.model,
          selectionSpecId: execution.specId,
        };
  const target = effect === "extract" ? execution : await defaultImageTarget(operation);
  return deriveImage(store, source, operation, { execution: target });
}
