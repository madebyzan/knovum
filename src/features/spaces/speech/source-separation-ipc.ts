import { invokeCommand } from "../../../foundation/ipc/invoke-command";
import type { RunObservation } from "../contracts/generated/RunObservation";
import type { SpeechMediaSource } from "../contracts/generated/SpeechMediaSource";
import type { SpeechSourceSeparationPublication } from "../contracts/generated/SpeechSourceSeparationPublication";
import type { StartSpeechSourceSeparationRequest } from "../contracts/generated/StartSpeechSourceSeparationRequest";

export function startSpeechSourceSeparation(request: StartSpeechSourceSeparationRequest) {
  return invokeCommand<RunObservation>("plugin:spaces|start_speech_source_separation", { request });
}

export function listSourceSeparations(spaceId: string, source: SpeechMediaSource) {
  return invokeCommand<SpeechSourceSeparationPublication[]>("plugin:spaces|list_source_separations", {
    spaceId,
    source,
  });
}

export function getLatestSourceSeparationRun(
  spaceId: string,
  sourceNodeId: string,
  source: SpeechMediaSource,
) {
  return invokeCommand<RunObservation | null>("plugin:spaces|get_latest_source_separation_run", {
    spaceId,
    sourceNodeId,
    source,
  });
}
