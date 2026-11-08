import type { SpeechMediaSource } from "../contracts/generated/SpeechMediaSource";
import type { SpeechSourceSeparationPublication } from "../contracts/generated/SpeechSourceSeparationPublication";
import type { VideoDubbingOutputConfig } from "../contracts/generated/VideoDubbingOutputConfig";

export function usesAutoVideoBackground(config: VideoDubbingOutputConfig): boolean {
  return (
    config.backgroundSeparation === null &&
    config.background !== null &&
    config.background.assetPath === config.video.assetPath &&
    config.background.sourceRevision === config.video.sourceRevision
  );
}

export function usesOriginalVideoBackground(config: VideoDubbingOutputConfig): boolean {
  return config.backgroundSeparation !== null || usesAutoVideoBackground(config);
}

export function applyDialogueIntervals(
  config: VideoDubbingOutputConfig,
  intervals: readonly { startMs: number; endMs: number }[],
): VideoDubbingOutputConfig {
  const sourceDialogueIntervals = intervals.map(({ startMs, endMs }) => ({ startMs, endMs }));
  return {
    ...config,
    translation: null,
    sourceDialogueIntervals,
    preserveOriginalOutsideDialogue:
      usesOriginalVideoBackground(config) && sourceDialogueIntervals.length > 0,
  };
}

export function selectBackground(
  config: VideoDubbingOutputConfig,
  background: SpeechMediaSource | null,
): VideoDubbingOutputConfig {
  const isAuto =
    background !== null &&
    background.assetPath === config.video.assetPath &&
    background.sourceRevision === config.video.sourceRevision;
  return {
    ...config,
    background: background ? { ...background } : null,
    backgroundSeparation: null,
    preserveOriginalOutsideDialogue: isAuto && config.sourceDialogueIntervals.length > 0,
  };
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
