import { formatDurationMs } from "./format-duration";
import type { SpaceTranscriptSegment } from "./schema/transcript";
import type { SpeakerBadge } from "./speaker-display";
import { formatSubtitleSrt } from "./subtitle-codec";

export function formatTranscriptPlainText(
  segments: readonly SpaceTranscriptSegment[],
  speakers?: ReadonlyMap<string, SpeakerBadge>,
): string {
  return segments
    .map((segment) => {
      const speaker = segment.speakerId ? speakers?.get(segment.speakerId)?.label : undefined;
      const prefix = speaker
        ? `[${formatDurationMs(segment.startMs)}] ${speaker}: `
        : `[${formatDurationMs(segment.startMs)}] `;
      return `${prefix}${segment.text.trim()}`;
    })
    .join("\n");
}

export function formatTranscriptSrt(
  segments: readonly SpaceTranscriptSegment[],
  speakers?: ReadonlyMap<string, SpeakerBadge>,
): string {
  return formatSubtitleSrt(
    segments.map((segment) => {
      const speaker = segment.speakerId ? speakers?.get(segment.speakerId)?.label : undefined;
      return {
        startMs: segment.startMs,
        endMs: segment.endMs,
        text: speaker ? `${speaker}: ${segment.text.trim()}` : segment.text.trim(),
      };
    }),
  );
}
