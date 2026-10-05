import type { VideoDubbingOutputConfig, VoiceLibraryRevision } from "./contracts";
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
  const voiceLanguages = new Map(input.voices.map((voice) => [voice.revisionId, voice.language]));
  return {
    sourceRevision: crypto.randomUUID(),
    pronunciationLexicon: input.lexicon,
    outputConfig: {
      ...currentConfig,
      voiceGainDb: Number(input.voiceGain),
      backgroundGainDb: Number(input.backgroundGain),
    },
    segments: input.lines.map((line) => {
      const language = voiceLanguages.get(line.voiceRevisionId);
      if (!language) throw new Error("A valid voice is required for every timed line");
      const speakerKey = line.speakerKey.trim();
      return {
        id: crypto.randomUUID(),
        text: line.text.trim(),
        ...(speakerKey ? { speakerKey } : {}),
        voiceRevisionId: line.voiceRevisionId,
        language,
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
