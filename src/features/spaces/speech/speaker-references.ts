import type { SpaceMediaTranscript } from "../media/schema/transcript";
import { DIALOGUE_PAUSE_SPLIT_THRESHOLD_MS } from "./video-transcript-splitter";

/** The shortest recording a voice can be cloned from. */
const MIN_REFERENCE_MS = 5_000;

export interface SpeakerReference {
  readonly speakerId: string;
  readonly startMs: number;
  readonly endMs: number;
}

interface Turn {
  speakerId: string;
  startMs: number;
  /** Where each of its segments ends: the places a selection can stop without cutting a word. */
  ends: number[];
}

/**
 * For each identified speaker, the selection to clone them from: their longest uninterrupted
 * turn — consecutive segments of theirs, no one else in between, no pause longer than a dialogue
 * pause — cut at a segment boundary to at most `maxMs`. A speaker who never talks for five seconds
 * on end has none. Speakers keep the order they first speak in.
 */
export function speakerReferences(transcript: SpaceMediaTranscript, maxMs: number): SpeakerReference[] {
  const best = new Map<string, SpeakerReference>();
  for (const turn of turns(transcript)) {
    const reference = selection(turn, maxMs);
    if (!reference) continue;
    const previous = best.get(turn.speakerId);
    if (!previous || length(reference) > length(previous)) best.set(turn.speakerId, reference);
  }
  const order = [...new Set(transcript.segments.flatMap((segment) => segment.speakerId ?? []))];
  return order.flatMap((speakerId) => best.get(speakerId) ?? []);
}

function turns(transcript: SpaceMediaTranscript): Turn[] {
  const segments = transcript.segments.toSorted((left, right) => left.startMs - right.startMs);
  const result: Turn[] = [];
  let current: Turn | undefined;
  for (const segment of segments) {
    const continues =
      current &&
      segment.speakerId === current.speakerId &&
      segment.startMs - current.ends.at(-1)! <= DIALOGUE_PAUSE_SPLIT_THRESHOLD_MS;
    if (current && continues) {
      current.ends.push(segment.endMs);
      continue;
    }
    current = segment.speakerId
      ? { speakerId: segment.speakerId, startMs: segment.startMs, ends: [segment.endMs] }
      : undefined;
    if (current) result.push(current);
  }
  return result;
}

function selection(turn: Turn, maxMs: number): SpeakerReference | undefined {
  const limit = turn.startMs + maxMs;
  const endMs = turn.ends.findLast((end) => end <= limit) ?? limit;
  const reference = { speakerId: turn.speakerId, startMs: turn.startMs, endMs };
  return length(reference) >= MIN_REFERENCE_MS ? reference : undefined;
}

function length(reference: SpeakerReference) {
  return reference.endMs - reference.startMs;
}
