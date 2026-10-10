import type { ActionAvailability } from "../../../../shared/contracts/action-availability";
import type { ImageOperationCapability } from "../../contracts/generated/ImageOperationCapability";
import type { LayerAvailability } from "../../knowledge-derivations/image-layers";

export type ImageActionReason =
  | "image_preparing"
  | "finish_image_edit"
  | "capability_failed"
  | "missing_credential"
  | "secret_database_unavailable"
  | "configuration_incomplete"
  | "provider_unsupported"
  | "credential_unavailable"
  | "upscale_source_budget"
  | "upscale_target_too_small"
  | "prompt_required";

export type ImageActionAvailability = ActionAvailability<ImageActionReason>;

export function imageToolAvailability(input: {
  sourceReady: boolean;
  editing: boolean;
  busy: boolean;
}): ImageActionAvailability {
  if (input.busy) return { state: "busy" };
  if (input.editing) return { state: "blocked", reason: "finish_image_edit" };
  if (!input.sourceReady) return { state: "blocked", reason: "image_preparing" };
  return { state: "ready" };
}

interface ImageActionAvailabilityInput {
  sourceReady: boolean;
  editing: boolean;
  capability: LayerAvailability | ImageOperationCapability | undefined;
  failed: boolean;
  busy: boolean;
}

export function imageActionAvailability(input: ImageActionAvailabilityInput): ImageActionAvailability {
  const local = imageToolAvailability(input);
  if (local.state !== "ready") return local;
  const { capability, failed } = input;
  if (failed) return { state: "blocked", reason: "capability_failed" };
  if (!capability) return { state: "checking" };
  return capability.state === "ready" || capability.state === "unverified" ? { state: "ready" } : capability;
}

export function upscaleAvailability(
  base: ImageActionAvailability,
  dimensions: { width: number; height: number } | undefined,
  target: number,
  budget: { maximumSide: number | null; maximumPixels: number | null },
): ImageActionAvailability {
  if (base.state !== "ready") return base;
  if (!dimensions) return { state: "blocked", reason: "image_preparing" };
  if (
    (budget.maximumSide !== null && Math.max(dimensions.width, dimensions.height) > budget.maximumSide) ||
    (budget.maximumPixels !== null && dimensions.width * dimensions.height > budget.maximumPixels)
  )
    return { state: "blocked", reason: "upscale_source_budget" };
  if (Math.max(dimensions.width, dimensions.height) >= target)
    return { state: "blocked", reason: "upscale_target_too_small" };
  return base;
}

export function promptedLayerAvailability(
  base: ImageActionAvailability,
  prompt: string,
): ImageActionAvailability {
  return base.state === "ready" && !prompt.trim() ? { state: "blocked", reason: "prompt_required" } : base;
}
