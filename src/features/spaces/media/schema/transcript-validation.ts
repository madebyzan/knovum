import type { z } from "zod";

import type { SpaceMediaTranscript, SpaceTranscriptSegment } from "./transcript";

export function validateMediaTranscript(transcript: SpaceMediaTranscript, context: z.RefinementCtx) {
  validateSegments(transcript, context);
  validateSubtitleCues(transcript, context);
  validateSpeakerTracks(transcript, context);
  if (transcript.text !== transcript.segments.map(({ text }) => text).join(" ")) {
    addIssue(context, ["text"], "Transcript text must match its ordered segments");
  }
}

function validateSegments(transcript: SpaceMediaTranscript, context: z.RefinementCtx) {
  const speakerIds = new Set(transcript.speakerTracks.map(({ speakerId }) => speakerId));
  const segmentIds = new Set<string>();
  const invalid = transcript.segments.some((segment, index, segments) => {
    const previous = segments[index - 1];
    const duplicateId = segmentIds.has(segment.id);
    segmentIds.add(segment.id);
    return (
      duplicateId ||
      segment.endMs <= segment.startMs ||
      segment.endMs > transcript.durationMs ||
      !hasCanonicalSampleOffsets(segment) ||
      (previous !== undefined &&
        (segment.startMs < previous.startMs || segment.startSample < previous.startSample)) ||
      (segment.speakerId !== null && !speakerIds.has(segment.speakerId)) ||
      unitsAreInvalid(segment)
    );
  });
  if (invalid) addIssue(context, ["segments"], "Transcript segments contain invalid normalized timing");
}

function hasCanonicalSampleOffsets(segment: SpaceTranscriptSegment) {
  return segment.startSample === segment.startMs * 16 && segment.endSample === segment.endMs * 16;
}

function unitsAreInvalid(segment: SpaceTranscriptSegment) {
  const characterCount = [...segment.text].length;
  const characters = [...segment.text];
  const unitIds = new Set<string>();
  return segment.units.some((unit, index, units) => {
    const previous = units[index - 1];
    const duplicateId = unitIds.has(unit.id);
    unitIds.add(unit.id);
    return (
      duplicateId ||
      unit.endCharacter <= unit.startCharacter ||
      unit.endCharacter > characterCount ||
      characters.slice(unit.startCharacter, unit.endCharacter).join("") !== unit.text ||
      (previous !== undefined && unit.startCharacter < previous.endCharacter) ||
      unitTimingIsInvalid(unit, segment)
    );
  });
}

function unitTimingIsInvalid(unit: SpaceTranscriptSegment["units"][number], segment: SpaceTranscriptSegment) {
  if (unit.startMs === null || unit.endMs === null) {
    return unit.startMs !== unit.endMs || unit.unalignedReason === null;
  }
  return (
    unit.endMs <= unit.startMs ||
    unit.startMs < segment.startMs ||
    unit.endMs > segment.endMs ||
    unit.unalignedReason !== null ||
    unit.unitKind === "segment"
  );
}

function validateSubtitleCues(transcript: SpaceMediaTranscript, context: z.RefinementCtx) {
  const invalid = transcript.subtitleCues.some(
    (cue, index, cues) =>
      cue.endMs <= cue.startMs ||
      cue.endMs > transcript.durationMs ||
      (index > 0 && cue.startMs < cues[index - 1]!.startMs),
  );
  if (invalid) addIssue(context, ["subtitleCues"], "Subtitle cues must be ordered and bounded");
}

function validateSpeakerTracks(transcript: SpaceMediaTranscript, context: z.RefinementCtx) {
  const ids = new Set<string>();
  for (const track of transcript.speakerTracks) {
    const invalidTurns = track.turns.some(
      (turn, index, turns) =>
        turn.endMs <= turn.startMs ||
        turn.endMs > transcript.durationMs ||
        (index > 0 && turn.startMs < turns[index - 1]!.endMs),
    );
    if (ids.has(track.speakerId) || invalidTurns) {
      addIssue(context, ["speakerTracks"], "Speaker tracks must be unique, ordered, and bounded");
      return;
    }
    ids.add(track.speakerId);
  }
}

function addIssue(context: z.RefinementCtx, path: Array<string | number>, message: string) {
  context.addIssue({ code: "custom", path, message });
}
