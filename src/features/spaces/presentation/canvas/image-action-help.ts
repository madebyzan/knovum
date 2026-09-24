import type { ImageActionAvailability, ImageActionReason } from "./image-action-availability";

type Translate = (key: string) => string;
export interface ImageActionHelp {
  label: string;
  actionLabel?: string;
}

const reasonKeys: Record<ImageActionReason, string> = {
  image_preparing: "spaces.media.imagePreparing",
  finish_image_edit: "spaces.media.finishImageEdit",
  capability_failed: "spaces.media.capabilityFailed",
  missing_credential: "spaces.media.layerKeyMissing",
  secret_database_unavailable: "spaces.media.secretDatabaseUnavailable",
  configuration_incomplete: "spaces.media.imageModelUnavailable",
  provider_unsupported: "spaces.media.imageProviderUnsupported",
  credential_unavailable: "spaces.media.imageCredentialUnavailable",
  upscale_min_size: "spaces.media.upscaleMinSize",
  upscale_target_too_small: "spaces.media.upscaleTargetTooSmall",
  prompt_required: "spaces.media.layerPromptRequired",
};

export function imageActionHelp(
  availability: ImageActionAvailability,
  t: Translate,
): ImageActionHelp | undefined {
  if (availability.state === "ready" || availability.state === "hidden") return undefined;
  if (availability.state === "checking") return { label: t("spaces.media.capabilityChecking") };
  if (availability.state === "busy") return { label: t("spaces.media.actionBusy") };
  const setup =
    availability.reason === "missing_credential" ||
    availability.reason === "configuration_incomplete" ||
    availability.reason === "provider_unsupported";
  return {
    label: t(reasonKeys[availability.reason]),
    ...(setup ? { actionLabel: t("spaces.media.openAiSettings") } : {}),
  };
}
