import type { SpaceMediaTranscript } from "./schema/transcript";

const SPEAKER_DOT_CLASSES = [
  "bg-blue-500",
  "bg-emerald-500",
  "bg-amber-500",
  "bg-violet-500",
  "bg-rose-500",
  "bg-cyan-500",
] as const;

export interface SpeakerBadge {
  id: string;
  label: string;
  dotClass: string;
}

export function buildSpeakerMap(
  speakerTracks: readonly SpaceMediaTranscript["speakerTracks"][number][] | undefined,
  formatSpeaker: (index: number) => string,
): ReadonlyMap<string, SpeakerBadge> {
  const map = new Map<string, SpeakerBadge>();
  if (!speakerTracks?.length) return map;
  speakerTracks.forEach((track, index) => {
    const numericSuffix = /^(?:SPEAKER_)?0*(\d+)$/iu.exec(track.speakerId);
    const label = numericSuffix ? formatSpeaker(Number(numericSuffix[1]) + 1) : track.speakerId;
    map.set(track.speakerId, {
      id: track.speakerId,
      label,
      dotClass: SPEAKER_DOT_CLASSES[index % SPEAKER_DOT_CLASSES.length]!,
    });
  });
  return map;
}
