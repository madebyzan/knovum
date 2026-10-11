import type { ImageExecutionTarget } from "../contracts/generated/ImageExecutionTarget";
import type { ImageSelection } from "../contracts/generated/ImageSelection";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import { deriveImage } from "./image-operation";

interface ImageSelectionDerivationInput {
  selection: ImageSelection;
  execution: Extract<ImageExecutionTarget, { type: "local" }>;
  effect: "extract" | "repair";
}

export async function deriveImageSelection(
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
  return deriveImage(source, operation, { execution: effect === "extract" ? execution : undefined });
}
