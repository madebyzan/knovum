import type { SpeechSourceSeparationCandidate } from "../contracts/generated/SpeechSourceSeparationCandidate";
import type { SpeechMediaSource, VideoDubbingOutputConfig } from "./contracts";

export function selectBackground(
  config: VideoDubbingOutputConfig,
  background: SpeechMediaSource | null,
): VideoDubbingOutputConfig {
  return { ...config, background, backgroundSeparation: null, preserveOriginalOutsideDialogue: false };
}

export function selectSeparatedBackground(
  config: VideoDubbingOutputConfig,
  candidate: SpeechSourceSeparationCandidate,
): VideoDubbingOutputConfig {
  return {
    ...config,
    background: { ...candidate.background },
    preserveOriginalOutsideDialogue: config.sourceDialogueIntervals.length > 0,
    backgroundSeparation: {
      separationId: candidate.separationId,
      modelSpecId: candidate.modelSpecId,
      source: { ...candidate.source },
    },
  };
}
