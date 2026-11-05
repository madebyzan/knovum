import type { ImageExecutionTarget } from "../contracts/generated/ImageExecutionTarget";
import type { ImageSelection } from "../contracts/generated/ImageSelection";
import { getImageOperationCapabilities, selectedImageTarget } from "./image-operation";
import type { ImageSelectionDerivationInput } from "./image-selection-derivation";

/**
 * The models a selection session chooses between: the selection models that draw the mask and the
 * repair models that paint the selected area away, each starting on the one chosen in Settings.
 */
export async function selectionChoices() {
  const { targets } = await getImageOperationCapabilities();
  const selection = targets.filter((target) => target.capability === "imageSelection");
  const repair = targets.filter((target) => target.capability === "imageInpainting");
  return {
    selection,
    repair,
    selectionModel: selectedImageTarget(selection),
    repairModel: repair.find((target) => target.selected && target.availability.state === "ready")?.execution,
  };
}

interface SelectionChoice {
  selection: ImageSelection;
  execution: ImageExecutionTarget | undefined;
  repair: ImageExecutionTarget | undefined;
}

/** What a draft derives for `effect`, or nothing while a model it needs is not chosen. */
export function selectionDerivation(
  { selection, execution, repair }: SelectionChoice,
  effect: ImageSelectionDerivationInput["effect"],
): ImageSelectionDerivationInput | undefined {
  if (execution?.type !== "local") return undefined;
  if (effect === "extract") return { effect, selection, selectionModel: execution };
  return repair?.type === "local"
    ? { effect, selection, selectionModel: execution, repairModel: repair }
    : undefined;
}
