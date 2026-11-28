import { sumBy } from "es-toolkit";

import { parseSubtitleCues } from "../media/subtitle-codec";

export interface DubbingSubtitle {
  startMs: number;
  endMs: number;
  text: string;
  speakerKey: string;
}

function validateCue(startMs: number, endMs: number, text: string, speakerKey: string) {
  if (
    !text ||
    [...text].length > 16_000 ||
    endMs <= startMs ||
    endMs > 43_200_000 ||
    !speakerKey ||
    [...speakerKey].length > 120
  )
    throw new Error("Subtitle cue exceeds dubbing limits");
}

export function parseDubbingSubtitles(source: string, speakerKey: string): DubbingSubtitle[] {
  const cues = parseSubtitleCues(source).map((parsed) => {
    const resolvedSpeaker = parsed.speaker ?? speakerKey;
    validateCue(parsed.startMs, parsed.endMs, parsed.text, resolvedSpeaker);
    return {
      startMs: parsed.startMs,
      endMs: parsed.endMs,
      text: parsed.text,
      speakerKey: resolvedSpeaker,
    };
  });
  if (!cues.length || cues.length > 10_000 || sumBy(cues, (item) => [...item.text].length) > 1_000_000)
    throw new Error("Subtitle script exceeds dubbing limits");
  return cues;
}
