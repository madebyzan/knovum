import { invokeCommand } from "../../../foundation/ipc/invoke-command";
import type { ReviewSpeechSourceSeparationRequest } from "../contracts/generated/ReviewSpeechSourceSeparationRequest";
import type { RunObservation } from "../contracts/generated/RunObservation";
import type { SpeechMediaSource } from "../contracts/generated/SpeechMediaSource";
import type { SpeechSourceSeparationCandidate } from "../contracts/generated/SpeechSourceSeparationCandidate";
import type { StartSpeechSourceSeparationRequest } from "../contracts/generated/StartSpeechSourceSeparationRequest";

export function startSpeechSourceSeparation(request: StartSpeechSourceSeparationRequest) {
  return invokeCommand<RunObservation>("plugin:spaces|start_speech_source_separation", { request });
}

export function getSpeechSourceSeparation(separationId: string) {
  return invokeCommand<SpeechSourceSeparationCandidate>("plugin:spaces|get_speech_source_separation", {
    separationId,
  });
}

export function reviewSpeechSourceSeparation(request: ReviewSpeechSourceSeparationRequest) {
  return invokeCommand<SpeechSourceSeparationCandidate>("plugin:spaces|review_speech_source_separation", {
    request,
  });
}

export function listAcceptedSourceSeparations(spaceId: string, source: SpeechMediaSource) {
  return invokeCommand<SpeechSourceSeparationCandidate[]>("plugin:spaces|list_accepted_source_separations", {
    spaceId,
    source,
  });
}

export function getLatestSourceSeparationRun(spaceId: string, source: SpeechMediaSource) {
  return invokeCommand<RunObservation | null>("plugin:spaces|get_latest_source_separation_run", {
    spaceId,
    source,
  });
}
