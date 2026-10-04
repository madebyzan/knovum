import type { SpeechSourceSeparationCandidate } from "../contracts/generated/SpeechSourceSeparationCandidate";
import type { SpeechMediaSource, VideoDubbingOutputConfig } from "./contracts";

export function selectBackground(
  config: VideoDubbingOutputConfig,
  background: SpeechMediaSource | null,
): VideoDubbingOutputConfig {
  return { ...config, background, backgroundSeparation: null };
}

export function selectSeparatedBackground(
  config: VideoDubbingOutputConfig,
  candidate: SpeechSourceSeparationCandidate,
): VideoDubbingOutputConfig {
  return {
    ...config,
    background: { ...candidate.background },
    backgroundSeparation: {
      separationId: candidate.separationId,
      modelSpecId: candidate.modelSpecId,
      source: { ...candidate.source },
    },
  };
}
