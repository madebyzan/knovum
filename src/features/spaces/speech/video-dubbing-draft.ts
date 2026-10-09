import type { SpeechSegment } from "../contracts/generated/SpeechSegment";
import type { VideoTranslation } from "../contracts/generated/VideoTranslation";
import type { VideoTranslationSegmentInput } from "../contracts/generated/VideoTranslationSegmentInput";
import type { VoiceLibraryRevision } from "../contracts/generated/VoiceLibraryRevision";
import { deliveryInstructionIsValid } from "./voice-options";

export interface VideoLineDraft {
  key: string;
  text: string;
  speakerKey: string;
  voiceRevisionId: string;
  deliveryInstruction: string;
  startSeconds: string;
  endSeconds: string;
  needsTimingEdit: boolean;
  sourceSegmentId?: string;
  sourceSegmentRevision?: number;
  sourceText?: string;
}

export function milliseconds(seconds: string) {
  return Math.round(Number(seconds) * 1000);
}

export function bindVideoSpeaker(
  lines: readonly VideoLineDraft[],
  speakerKey: string,
  voiceRevisionId: string,
  supportsDeliveryInstruction: boolean,
) {
  return lines.map((line) =>
    line.speakerKey.trim() === speakerKey
      ? {
          ...line,
          voiceRevisionId,
          deliveryInstruction: supportsDeliveryInstruction ? line.deliveryInstruction : "",
        }
      : line,
  );
}

export function videoLineIsValid(
  line: VideoLineDraft,
  voices: readonly VoiceLibraryRevision[],
  durationMs: number,
) {
  const start = milliseconds(line.startSeconds);
  const end = milliseconds(line.endSeconds);
  return Boolean(
    line.text.trim() &&
    voices.some((voice) => voice.revisionId === line.voiceRevisionId) &&
    deliveryInstructionIsValid(voices, line.voiceRevisionId, line.deliveryInstruction) &&
    line.startSeconds.trim() &&
    line.endSeconds.trim() &&
    validVideoWindow(start, end, durationMs),
  );
}

export function videoLineFromSegment(segment: SpeechSegment): VideoLineDraft {
  return {
    key: segment.id,
    text: segment.text,
    speakerKey: segment.speakerKey ?? "",
    voiceRevisionId: segment.voiceRevisionId,
    deliveryInstruction: segment.deliveryInstruction ?? "",
    startSeconds: String((segment.targetStartMs ?? 0) / 1000),
    endSeconds: String((segment.targetEndMs ?? 0) / 1000),
    needsTimingEdit: segment.state === "needs_edit",
    ...(segment.sourceSegmentId === undefined ? {} : { sourceSegmentId: segment.sourceSegmentId }),
    ...(segment.sourceSegmentRevision === undefined
      ? {}
      : { sourceSegmentRevision: segment.sourceSegmentRevision }),
    ...(segment.sourceText === undefined ? {} : { sourceText: segment.sourceText }),
  };
}

function validVideoWindow(startMs: number, endMs: number, durationMs: number) {
  return (
    Number.isSafeInteger(startMs) &&
    Number.isSafeInteger(endMs) &&
    startMs >= 0 &&
    endMs > startMs &&
    endMs <= durationMs
  );
}

export function translationInputs(
  lines: readonly VideoLineDraft[],
  alreadyTranslated: boolean,
): VideoTranslationSegmentInput[] {
  if (lines.some((line) => !line.text.trim())) return [];
  return lines.map((line) => {
    const text = alreadyTranslated ? (line.sourceText ?? line.text.trim()) : line.text.trim();
    const sourceChanged =
      !alreadyTranslated && line.sourceText !== undefined && line.sourceText.trim() !== text;
    const revision = (line.sourceSegmentRevision ?? 1) + (sourceChanged ? 1 : 0);
    if (revision > 0xffff_ffff) throw new Error("Video source segment revision is exhausted");
    return { sourceSegmentId: line.sourceSegmentId ?? line.key, sourceSegmentRevision: revision, text };
  });
}

export function applyTranslation(
  lines: readonly VideoLineDraft[],
  translation: VideoTranslation,
  alreadyTranslated: boolean,
): VideoLineDraft[] {
  const inputs = translationInputs(lines, alreadyTranslated);
  if (
    inputs.length !== translation.segments.length ||
    inputs.some((input, index) => {
      const output = translation.segments[index];
      return (
        !output ||
        output.sourceSegmentId !== input.sourceSegmentId ||
        output.sourceSegmentRevision !== input.sourceSegmentRevision ||
        output.sourceText !== input.text.trim()
      );
    })
  )
    throw new Error("Translation result does not match the current video script");
  const translationBySourceSegmentId = new Map(
    translation.segments.map((segment) => [segment.sourceSegmentId, segment]),
  );
  if (translationBySourceSegmentId.size !== lines.length) {
    throw new Error("Translation result does not match the current video script");
  }
  return lines.map((line) => {
    const sourceSegmentId = line.sourceSegmentId ?? line.key;
    const translated = translationBySourceSegmentId.get(sourceSegmentId);
    if (!translated) throw new Error("Translation result does not match the current video script");
    return {
      ...line,
      text: translated.translatedText,
      sourceSegmentId,
      sourceSegmentRevision: translated.sourceSegmentRevision,
      sourceText: translated.sourceText,
      needsTimingEdit: false,
    };
  });
}
