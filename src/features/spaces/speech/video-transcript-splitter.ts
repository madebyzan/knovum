import type { SpaceMediaTranscript, SpaceTranscriptSegment } from "../media/schema/transcript";
import type { VideoLineDraft } from "./video-dubbing-draft";

export const DIALOGUE_PAUSE_SPLIT_THRESHOLD_MS = 1200;

export interface ImportedTranscriptDialogue {
  lines: VideoLineDraft[];
  intervals: Array<{ startMs: number; endMs: number }>;
}

type TimedUnit = SpaceTranscriptSegment["units"][number] & { startMs: number; endMs: number };

interface LineDraftParams {
  text: string;
  speakerKey: string;
  voiceRevisionId: string;
  startMs: number;
  endMs: number;
  sourceSegmentId: string;
  sourceSegmentRevision: number;
  sourceText?: string;
}

export function linesFromTranscript(
  transcript: SpaceMediaTranscript,
  defaultVoiceId: string,
  pauseThresholdMs = DIALOGUE_PAUSE_SPLIT_THRESHOLD_MS,
): ImportedTranscriptDialogue {
  const lines: VideoLineDraft[] = [];
  const intervals: Array<{ startMs: number; endMs: number }> = [];
  for (const segment of transcript.segments) {
    const outcome = splitSegmentToLines(segment, defaultVoiceId, pauseThresholdMs);
    lines.push(...outcome.lines);
    intervals.push(...outcome.intervals);
  }
  return { lines, intervals };
}

function splitSegmentToLines(
  segment: SpaceTranscriptSegment,
  defaultVoiceId: string,
  pauseThresholdMs: number,
): ImportedTranscriptDialogue {
  const timedUnits = extractTimedUnits(segment.units);
  if (!timedUnits) {
    return singleSegmentLine(segment, defaultVoiceId, segment.startMs, segment.endMs);
  }
  const clusters = clusterTimedUnits(timedUnits, pauseThresholdMs);
  if (clusters.length <= 1) {
    const startMs = resolveBoundary(segment.startMs, timedUnits[0]!.startMs, pauseThresholdMs, true);
    const endMs = resolveBoundary(
      segment.endMs,
      timedUnits[timedUnits.length - 1]!.endMs,
      pauseThresholdMs,
      false,
    );
    return singleSegmentLine(segment, defaultVoiceId, startMs, endMs);
  }
  return splitClustersToLines(segment, clusters, defaultVoiceId, pauseThresholdMs);
}

function extractTimedUnits(units: SpaceTranscriptSegment["units"]): TimedUnit[] | null {
  if (!units.length) return null;
  const timed: TimedUnit[] = [];
  for (const unit of units) {
    if (unit.startMs === null || unit.endMs === null || unit.endMs <= unit.startMs) return null;
    timed.push(unit as TimedUnit);
  }
  return timed;
}

function clusterTimedUnits(units: TimedUnit[], thresholdMs: number): TimedUnit[][] {
  const clusters: TimedUnit[][] = [];
  let current: TimedUnit[] = [units[0]!];
  for (let i = 1; i < units.length; i++) {
    const prev = units[i - 1]!;
    const unit = units[i]!;
    if (unit.startMs - prev.endMs >= thresholdMs) {
      clusters.push(current);
      current = [unit];
    } else {
      current.push(unit);
    }
  }
  clusters.push(current);
  return clusters;
}

function splitClustersToLines(
  segment: SpaceTranscriptSegment,
  clusters: TimedUnit[][],
  defaultVoiceId: string,
  thresholdMs: number,
): ImportedTranscriptDialogue {
  const characters = Array.from(segment.text);
  const lines: VideoLineDraft[] = [];
  const intervals: Array<{ startMs: number; endMs: number }> = [];
  for (let j = 0; j < clusters.length; j++) {
    const cluster = clusters[j]!;
    const firstUnit = cluster[0]!;
    const lastUnit = cluster[cluster.length - 1]!;
    const startChar = j === 0 ? 0 : firstUnit.startCharacter;
    const endChar = j === clusters.length - 1 ? characters.length : clusters[j + 1]![0]!.startCharacter;
    const lineText = characters.slice(startChar, endChar).join("").trim();
    if (!lineText) continue;

    const startMs =
      j === 0 ? resolveBoundary(segment.startMs, firstUnit.startMs, thresholdMs, true) : firstUnit.startMs;
    const rawEnd =
      j === clusters.length - 1
        ? resolveBoundary(segment.endMs, lastUnit.endMs, thresholdMs, false)
        : lastUnit.endMs;
    const endMs = Math.max(rawEnd, startMs + 100);

    lines.push(
      buildLineDraft({
        text: lineText,
        speakerKey: segment.speakerId ?? "",
        voiceRevisionId: defaultVoiceId,
        startMs,
        endMs,
        sourceSegmentId: `${segment.id}:${j}`,
        sourceSegmentRevision: segment.revision,
      }),
    );
    intervals.push({ startMs, endMs });
  }
  return { lines, intervals };
}

function resolveBoundary(
  segmentTime: number,
  unitTime: number,
  thresholdMs: number,
  isStart: boolean,
): number {
  const diff = isStart ? unitTime - segmentTime : segmentTime - unitTime;
  return diff < thresholdMs ? segmentTime : unitTime;
}

function singleSegmentLine(
  segment: SpaceTranscriptSegment,
  voiceRevisionId: string,
  startMs: number,
  endMs: number,
): ImportedTranscriptDialogue {
  const boundedEnd = Math.max(endMs, startMs + 100);
  const line = buildLineDraft({
    text: segment.text,
    speakerKey: segment.speakerId ?? "",
    voiceRevisionId,
    startMs,
    endMs: boundedEnd,
    sourceSegmentId: segment.id,
    sourceSegmentRevision: segment.revision,
    sourceText: segment.text,
  });
  return { lines: [line], intervals: [{ startMs, endMs: boundedEnd }] };
}

function buildLineDraft(params: LineDraftParams): VideoLineDraft {
  return {
    key: crypto.randomUUID(),
    text: params.text,
    speakerKey: params.speakerKey,
    voiceRevisionId: params.voiceRevisionId,
    deliveryInstruction: "",
    startSeconds: String(Number((params.startMs / 1000).toFixed(3))),
    endSeconds: String(Number((params.endMs / 1000).toFixed(3))),
    sourceSegmentId: params.sourceSegmentId,
    sourceSegmentRevision: params.sourceSegmentRevision,
    sourceText: params.sourceText ?? params.text,
  };
}
