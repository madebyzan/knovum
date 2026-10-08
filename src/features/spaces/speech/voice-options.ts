import type { VoiceLibraryRevision } from "../contracts/generated/VoiceLibraryRevision";

export const voiceEngineLabels: Readonly<Record<string, string>> = {
  moss_nano: "MOSS-TTS-Nano",
  voxcpm2: "VoxCPM2",
};

function activeVoiceRevisions(voices: readonly VoiceLibraryRevision[]) {
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

export function voiceSupportsDeliveryInstruction(
  voices: readonly VoiceLibraryRevision[],
  revisionId: string,
) {
  return Boolean(voices.find((voice) => voice.revisionId === revisionId)?.supportsDeliveryInstruction);
}

export function deliveryInstructionIsValid(
  voices: readonly VoiceLibraryRevision[],
  revisionId: string,
  value: string,
) {
  const instruction = value.trim();
  return (
    [...instruction].length <= 500 && (!instruction || voiceSupportsDeliveryInstruction(voices, revisionId))
  );
}
