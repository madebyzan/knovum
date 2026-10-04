import type { VoiceProfileSummary } from "./contracts";

export function activeVoiceProfiles(voices: readonly VoiceProfileSummary[]) {
  return voices.filter((voice) => voice.isActive && voice.archivedAt == null);
}

export function voiceRevisionOptions(
  voices: readonly VoiceProfileSummary[],
  selectedRevisionIds: Iterable<string>,
) {
  const selected = new Set(selectedRevisionIds);
  return voices
    .filter((voice) => (voice.isActive && voice.archivedAt == null) || selected.has(voice.activeRevisionId))
    .map((voice) => ({ value: voice.activeRevisionId, label: voice.name }));
}

export function defaultVoiceRevisionId(voices: readonly VoiceProfileSummary[]) {
  return activeVoiceProfiles(voices)[0]?.activeRevisionId ?? "";
}
