import type { VoiceLibraryRevision } from "../contracts/generated/VoiceLibraryRevision";

export function activeVoiceRevisions(voices: readonly VoiceLibraryRevision[]) {
  return voices.filter((voice) => voice.isActive && voice.archivedAt == null);
}

export function voiceRevisionOptions(
  voices: readonly VoiceLibraryRevision[],
  selectedRevisionIds: Iterable<string>,
) {
  const selected = new Set(selectedRevisionIds);
  return voices
    .filter((voice) => (voice.isActive && voice.archivedAt == null) || selected.has(voice.revisionId))
    .map((voice) => ({
      value: voice.revisionId,
      label: `${voice.name} · v${voice.revision} · ${voice.language}`,
    }));
}

export function defaultVoiceRevisionId(voices: readonly VoiceLibraryRevision[]) {
  return activeVoiceRevisions(voices)[0]?.revisionId ?? "";
}
