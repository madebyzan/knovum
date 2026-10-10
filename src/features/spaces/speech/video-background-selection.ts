import type { SpeechMediaSource } from "../contracts/generated/SpeechMediaSource";
import type { SpeechSourceSeparationPublication } from "../contracts/generated/SpeechSourceSeparationPublication";
import type { VideoDubbingOutputConfig } from "../contracts/generated/VideoDubbingOutputConfig";

export function selectBackground(
  config: VideoDubbingOutputConfig,
  background: SpeechMediaSource | null,
): VideoDubbingOutputConfig {
  return { ...config, background, backgroundSeparation: null, preserveOriginalOutsideDialogue: false };
}

export function selectSeparatedBackground(
  config: VideoDubbingOutputConfig,
  publication: SpeechSourceSeparationPublication,
): VideoDubbingOutputConfig {
  return {
    ...config,
    background: { ...publication.background },
    preserveOriginalOutsideDialogue: config.sourceDialogueIntervals.length > 0,
    backgroundSeparation: {
      separationId: publication.separationId,
      modelSpecId: publication.modelSpecId,
      source: { ...publication.source },
    },
  };
}
