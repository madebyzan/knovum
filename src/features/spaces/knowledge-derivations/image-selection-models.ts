import { queryClient } from "../../../foundation/query/query-client";
import { localModelsQueryOptions } from "../../../foundation/runtime/use-local-models-query";
import type { ImageExecutionTarget } from "../contracts/generated/ImageExecutionTarget";
import { preferredVerifiedImageTarget, verifiedImageTargetOptions } from "./image-model-targets";
import { getImageOperationCapabilities, selectedImageTarget } from "./image-operation";

export type LocalImageTarget = Extract<ImageExecutionTarget, { type: "local" }>;

/** Pin the Settings-selected models for this session; preview and publication read the same mask. */
export async function loadSelectionModels() {
  const [capabilities, models] = await Promise.all([
    getImageOperationCapabilities(),
    queryClient.fetchQuery({ ...localModelsQueryOptions, staleTime: 0, retry: false }),
  ]);
  const targets = capabilities.targets;
  const chosen = preferredVerifiedImageTarget(verifiedImageTargetOptions(targets, models, "imageSelection"));
  const selection =
    chosen?.execution ??
    selectedImageTarget(targets.filter((target) => target.capability === "imageSelection"));
  if (selection.type !== "local") throw new Error("Image selection requires a local model");
  const repair = targets.find(
    (target) =>
      target.capability === "imageInpainting" && target.selected && target.availability.state === "ready",
  )?.execution;
  return { selection, repair: repair?.type === "local" ? repair : undefined };
}
