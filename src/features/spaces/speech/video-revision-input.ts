import type { VideoDubbingOutputConfig } from "../contracts/generated/VideoDubbingOutputConfig";
import type { VoiceLibraryRevision } from "../contracts/generated/VoiceLibraryRevision";
import { milliseconds, type VideoLineDraft } from "./video-dubbing-draft";

export function videoRevisionInput(input: {
  config: VideoDubbingOutputConfig | undefined;
  lines: readonly VideoLineDraft[];
  voices: readonly VoiceLibraryRevision[];
  lexicon: Record<string, string>;
  voiceGain: string;
  backgroundGain: string;
}) {
  const currentConfig = input.config;
  if (!currentConfig) throw new Error("Video dubbing output configuration is unavailable");
  const voiceByRevisionId = new Map(input.voices.map((voice) => [voice.revisionId, voice]));
  return {
    sourceRevision: crypto.randomUUID(),
    pronunciationLexicon: input.lexicon,
    outputConfig: {
      ...currentConfig,
      voiceGainDb: Number(input.voiceGain),
      backgroundGainDb: Number(input.backgroundGain),
    },
    segments: input.lines.map((line) => {
      const voice = voiceByRevisionId.get(line.voiceRevisionId);
      if (!voice) throw new Error("A valid voice is required for every timed line");
      const speakerKey = line.speakerKey.trim();
      const deliveryInstruction = line.deliveryInstruction.trim();
      return {
        id: crypto.randomUUID(),
        text: line.text.trim(),
        ...(speakerKey ? { speakerKey } : {}),
        voiceRevisionId: line.voiceRevisionId,
        language: voice.language,
        ...(deliveryInstruction ? { deliveryInstruction } : {}),
        targetStartMs: milliseconds(line.startSeconds),
        targetEndMs: milliseconds(line.endSeconds),
        ...(line.sourceSegmentId && line.sourceSegmentRevision && line.sourceText
          ? {
              sourceSegmentId: line.sourceSegmentId,
              sourceSegmentRevision: line.sourceSegmentRevision,
              sourceText: line.sourceText,
            }
          : {}),
      };
    }),
  };
}
