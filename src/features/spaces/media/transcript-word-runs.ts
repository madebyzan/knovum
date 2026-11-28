import type { SpaceTranscriptSegment } from "./schema/transcript";

export type TranscriptCueRun =
  | { kind: "gap"; key: string; text: string }
  | { kind: "word"; key: string; text: string; startMs: number | null; endMs: number | null };

/**
 * Slices a transcript segment into interactive timed word runs while preserving every space,
 * newline, and punctuation mark from `segment.text` verbatim via character offsets.
 */
export function buildTranscriptWordRuns(segment: SpaceTranscriptSegment): TranscriptCueRun[] | null {
  const units = segment.units;
  if (units.length < 2) return null;
  if (!units.some((unit) => unit.startMs !== null && unit.endMs !== null)) return null;

  const chars = [...segment.text];
  const runs: TranscriptCueRun[] = [];
  let cursor = 0;

  for (let index = 0; index < units.length; index++) {
    const unit = units[index]!;
    const start = Math.max(cursor, Math.min(unit.startCharacter, chars.length));
    const end = Math.max(start, Math.min(unit.endCharacter, chars.length));
    if (start > cursor) {
      runs.push({ kind: "gap", key: `gap:${index}`, text: chars.slice(cursor, start).join("") });
    }
    const text = end > start ? chars.slice(start, end).join("") : unit.text;
    if (text) {
      runs.push({ kind: "word", key: unit.id, text, startMs: unit.startMs, endMs: unit.endMs });
    }
    cursor = end;
  }

  if (cursor < chars.length) {
    runs.push({ kind: "gap", key: "gap:tail", text: chars.slice(cursor).join("") });
  }
  return runs.length ? runs : null;
}
