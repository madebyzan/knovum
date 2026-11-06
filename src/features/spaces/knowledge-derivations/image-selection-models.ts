import type { ImageExecutionTarget } from "../contracts/generated/ImageExecutionTarget";
import { getImageOperationCapabilities, selectedImageTarget } from "./image-operation";

export type LocalImageTarget = Extract<ImageExecutionTarget, { type: "local" }>;

/** Pin the Settings-selected models for this session; preview and publication read the same mask. */
export async function loadSelectionModels() {
  const { targets } = await getImageOperationCapabilities();
  const selection = selectedImageTarget(targets.filter((target) => target.capability === "imageSelection"));
  if (selection.type !== "local") throw new Error("Image selection requires a local model");
  const repair = targets.find(
    (target) =>
      target.capability === "imageInpainting" && target.selected && target.availability.state === "ready",
  )?.execution;
  return { selection, repair: repair?.type === "local" ? repair : undefined };
}
