import { invokeCommand } from "../../../foundation/ipc/invoke-command";
import type { RunObservation } from "../contracts/generated/RunObservation";
import type {
  CreateCandidateVoiceRequest,
  DraftVoiceCandidateRequest,
  VoiceCandidate,
  VoiceProfileSummary,
} from "./contracts";

export function draftVoiceCandidate(request: DraftVoiceCandidateRequest) {
  return invokeCommand<RunObservation>("plugin:spaces|draft_voice_candidate", { request });
}
export function getVoiceCandidate(candidateId: string, spaceId: string) {
  return invokeCommand<VoiceCandidate>("plugin:spaces|get_voice_candidate", { candidateId, spaceId });
}
export function getLatestVoiceCandidate(spaceId: string) {
  return invokeCommand<VoiceCandidate | null>("plugin:spaces|get_latest_voice_candidate", { spaceId });
}
export function getActiveVoiceCandidateRun(spaceId: string) {
  return invokeCommand<RunObservation | null>("plugin:spaces|get_active_voice_candidate_run", { spaceId });
}
export function markVoiceCandidateAuditioned(candidateId: string, spaceId: string) {
  return invokeCommand<VoiceCandidate>("plugin:spaces|mark_voice_candidate_auditioned", {
    candidateId,
    spaceId,
  });
}
export function discardVoiceCandidate(candidateId: string, spaceId: string) {
  return invokeCommand<void>("plugin:spaces|discard_voice_candidate", { candidateId, spaceId });
}
export function createCandidateVoice(request: CreateCandidateVoiceRequest) {
  return invokeCommand<VoiceProfileSummary>("plugin:spaces|create_candidate_voice", { request });
}
