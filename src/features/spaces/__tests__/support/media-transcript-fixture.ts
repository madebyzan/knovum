import type { SpaceMediaTranscript, SpaceTranscriptSegment } from "../../media/schema/transcript";

export function transcriptFixture(
  sourceRevision: string,
  parts: Array<Pick<SpaceTranscriptSegment, "startMs" | "endMs" | "text">>,
): SpaceMediaTranscript {
  return {
    id: "1".repeat(64),
    sourceRevision,
    revision: 1,
    language: "en",
    durationMs: parts.at(-1)?.endMs ?? 0,
    text: parts.map(({ text }) => text).join(" "),
    provenance: { engineId: "whisper", modelSpecId: "small", modelRevision: "fixed", timing: "engine_words" },
    speakerTracks: [],
    subtitleCues: parts.map(({ startMs, endMs, text }) => ({ startMs, endMs, text })),
    segments: parts.map((part, index) => ({
      startMs: part.startMs,
      endMs: part.endMs,
      text: part.text,
      id: String(index + 1).padStart(64, "0"),
      revision: 1,
      startSample: part.startMs * 16,
      endSample: part.endMs * 16,
      speakerId: null,
      units: [
        {
          id: (index + 1).toString(16).padStart(64, "b"),
          text: part.text,
          startCharacter: 0,
          endCharacter: [...part.text].length,
          startMs: part.startMs,
          endMs: part.endMs,
          score: 1,
          unalignedReason: null,
          unitKind: "word",
        },
      ],
    })),
  };
}
