import { convertFileSrc } from "@tauri-apps/api/core";

import type { VoiceCandidate, VoiceLibraryRevision } from "./contracts";
import {
  createCandidateVoice,
  discardVoiceCandidate,
  markVoiceCandidateAuditioned,
} from "./voice-candidate-ipc";

export type PlayableVoiceCandidate = VoiceCandidate & { source: string };

export function presentVoiceCandidate(candidate: VoiceCandidate): PlayableVoiceCandidate {
  return { ...candidate, source: convertFileSrc(candidate.audio.path) };
}

export async function auditionVoiceCandidate(candidate: VoiceCandidate) {
  return presentVoiceCandidate(await markVoiceCandidateAuditioned(candidate.candidateId, candidate.spaceId));
}

export function selectVoiceCandidate(
  candidate: VoiceCandidate,
  name: string,
  authorized: boolean,
): Promise<VoiceLibraryRevision> {
  return createCandidateVoice({
    candidateId: candidate.candidateId,
    spaceId: candidate.spaceId,
    name,
    authorized,
  });
}

export function rejectVoiceCandidate(candidate: VoiceCandidate) {
  return discardVoiceCandidate(candidate.candidateId, candidate.spaceId);
}
