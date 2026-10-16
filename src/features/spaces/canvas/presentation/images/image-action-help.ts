import { i18n } from "../../../../../foundation/i18n/instance";
import type { ImageActionAvailability, ImageActionReason } from "./image-action-availability";

export interface ImageActionHelp {
  label: string;
  actionLabel?: string | undefined;
}

const REASON_KEYS: Record<ImageActionReason, string> = {
  image_preparing: "spaces.media.imageEditor.status.imagePreparing",
  finish_image_edit: "spaces.media.imageEditor.finishImageEdit",
  capability_failed: "spaces.media.imageEditor.errors.capabilityCheckFailed",
  configuration_incomplete: "spaces.media.imageEditor.errors.modelUnavailable",
  provider_unsupported: "spaces.media.imageEditor.errors.providerUnsupported",
  credential_unavailable: "spaces.media.imageEditor.errors.credentialUnavailable",
  upscale_source_budget: "spaces.media.imageEditor.upscale.sourceBudget",
  upscale_target_too_small: "spaces.media.imageEditor.upscale.targetTooSmall",
};

export function imageActionHelp(availability: ImageActionAvailability): ImageActionHelp | undefined {
  const t = i18n.global.t;

  if (availability.state === "ready" || availability.state === "hidden") return undefined;
  if (availability.state === "checking")
    return { label: t("spaces.media.imageEditor.status.capabilityChecking") };
  if (availability.state === "busy") return { label: t("spaces.media.imageEditor.errors.busy") };
  const setup =
    availability.reason === "configuration_incomplete" || availability.reason === "provider_unsupported";
  return {
    label: t(REASON_KEYS[availability.reason]),
    ...(setup ? { actionLabel: t("spaces.media.imageEditor.openAiSettings") } : {}),
  };
}
