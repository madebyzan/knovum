import type { VideoTranslationResult } from "../contracts/generated/VideoTranslationResult";
import type { VideoTranslationSegmentInput } from "../contracts/generated/VideoTranslationSegmentInput";
import type { SpeechSegment } from "./contracts";

export interface VideoLineDraft {
  key: string;
  text: string;
  speakerKey: string;
  voiceRevisionId: string;
  startSeconds: string;
  endSeconds: string;
  needsTimingEdit: boolean;
  sourceSegmentId?: string;
  sourceSegmentRevision?: number;
  sourceText?: string;
}

export const milliseconds = (seconds: string) => Math.round(Number(seconds) * 1000);

export function videoLineFromSegment(segment: SpeechSegment): VideoLineDraft {
  return {
    key: segment.id,
    text: segment.text,
    speakerKey: segment.speakerKey ?? "",
    voiceRevisionId: segment.voiceRevisionId,
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

export function validVideoWindow(start: number, end: number, duration: number) {
  return (
    Number.isSafeInteger(start) && Number.isSafeInteger(end) && start >= 0 && end > start && end <= duration
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
  result: VideoTranslationResult,
  alreadyTranslated: boolean,
): VideoLineDraft[] {
  const inputs = translationInputs(lines, alreadyTranslated);
  if (
    inputs.length !== result.segments.length ||
    inputs.some((input, index) => {
      const output = result.segments[index];
      return (
        !output ||
        output.sourceSegmentId !== input.sourceSegmentId ||
        output.sourceSegmentRevision !== input.sourceSegmentRevision ||
        output.sourceText !== input.text.trim()
      );
    })
  )
    throw new Error("Translation result does not match the current video script");
  const bySource = new Map(result.segments.map((segment) => [segment.sourceSegmentId, segment]));
  if (bySource.size !== lines.length) {
    throw new Error("Translation result does not match the current video script");
  }
  return lines.map((line) => {
    const sourceSegmentId = line.sourceSegmentId ?? line.key;
    const translated = bySource.get(sourceSegmentId);
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
