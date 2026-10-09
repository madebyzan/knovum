import { invokeCommand } from "../../../foundation/ipc/invoke-command";
import type { CreateCandidateVoiceRequest } from "../contracts/generated/CreateCandidateVoiceRequest";
import type { DraftVoiceCandidateRequest } from "../contracts/generated/DraftVoiceCandidateRequest";
import type { RunObservation } from "../contracts/generated/RunObservation";
import type { VoiceCandidate } from "../contracts/generated/VoiceCandidate";
import type { VoiceLibraryRevision } from "../contracts/generated/VoiceLibraryRevision";

export function draftVoiceCandidate(request: DraftVoiceCandidateRequest) {
  return invokeCommand<RunObservation>("plugin:spaces|draft_voice_candidate", { request });
}
export function getVoiceCandidate(candidateId: string) {
  return invokeCommand<VoiceCandidate>("plugin:spaces|get_voice_candidate", { candidateId });
}
export function listVoiceCandidates() {
  return invokeCommand<VoiceCandidate[]>("plugin:spaces|list_voice_candidates");
}
export function getActiveVoiceCandidateRun() {
  return invokeCommand<RunObservation | null>("plugin:spaces|get_active_voice_candidate_run");
}
export function markVoiceCandidateAuditioned(candidateId: string) {
  return invokeCommand<VoiceCandidate>("plugin:spaces|mark_voice_candidate_auditioned", { candidateId });
}
export function discardVoiceCandidate(candidateId: string) {
  return invokeCommand<void>("plugin:spaces|discard_voice_candidate", { candidateId });
}
export function createCandidateVoice(request: CreateCandidateVoiceRequest) {
  return invokeCommand<VoiceLibraryRevision>("plugin:spaces|create_candidate_voice", { request });
}
