import type { ImageExecutionTarget } from "../contracts/generated/ImageExecutionTarget";
import type { ImageSelection } from "../contracts/generated/ImageSelection";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import { deriveImage } from "./image-operation";

type LocalImageTarget = Extract<ImageExecutionTarget, { type: "local" }>;

/**
 * Cutting a selection out runs on the selection model alone; repairing it away also paints the
 * area, so it names the repair model that paints it.
 */
export type ImageSelectionDerivationInput =
  | { effect: "extract"; selection: ImageSelection; selectionModel: LocalImageTarget }
  | {
      effect: "repair";
      selection: ImageSelection;
      selectionModel: LocalImageTarget;
      repairModel: LocalImageTarget;
    };

export async function deriveImageSelection(source: SpaceCanvasNode, input: ImageSelectionDerivationInput) {
  const { selection, selectionModel } = input;
  if (input.effect === "extract")
    return deriveImage(source, { kind: "extract-selection", selection }, { execution: selectionModel });
  return deriveImage(
    source,
    {
      kind: "remove-selected-object",
      selection,
      selectionModel: selectionModel.model,
      selectionSpecId: selectionModel.specId,
    },
    { execution: input.repairModel },
  );
}
