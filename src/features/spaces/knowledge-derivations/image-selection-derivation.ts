import type { ImageSelection } from "../contracts/generated/ImageSelection";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import { deriveImage } from "./image-operation";
import type { LocalImageTarget } from "./image-selection-models";

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

/** Build only the requested effect; extracting has no repair-model dependency. */
export function selectionDerivationInput(
  draft: {
    selection: ImageSelection;
    execution: LocalImageTarget | undefined;
    repair: LocalImageTarget | undefined;
  },
  effect: ImageSelectionDerivationInput["effect"],
): ImageSelectionDerivationInput | undefined {
  if (!draft.execution) return undefined;
  const input = { selection: draft.selection, selectionModel: draft.execution };
  if (effect === "extract") return { ...input, effect };
  return draft.repair ? { ...input, effect, repairModel: draft.repair } : undefined;
}

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
