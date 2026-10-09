import { sumBy } from "es-toolkit";

export interface DubbingSubtitle {
  startMs: number;
  endMs: number;
  text: string;
  speakerKey: string;
}

function timestamp(value: string) {
  const match = /^(?:(\d{2,}):)?(\d{2}):(\d{2})[.,](\d{3})$/u.exec(value);
  if (!match || Number(match[2]) >= 60 || Number(match[3]) >= 60)
    throw new Error("Subtitle timestamp is invalid");
  return (Number(match[1] ?? 0) * 3600 + Number(match[2]) * 60 + Number(match[3])) * 1000 + Number(match[4]);
}

function cue(block: string, speakerKey: string): DubbingSubtitle {
  const lines = block.split("\n");
  const timingIndex = lines[0]?.includes(" --> ") ? 0 : 1;
  const timing = lines[timingIndex]?.match(/^(\S+)\s+-->\s+(\S+)(?:\s+.*)?$/u);
  if (!timing) throw new Error("Subtitle cue has no valid timing");
  const startMs = timestamp(timing[1]!);
  const endMs = timestamp(timing[2]!);
  const content = lines
    .slice(timingIndex + 1)
    .join("\n")
    .replace(/<v\s+([^>]+)>/gu, (_, speaker: string) => {
      speakerKey = speaker.trim();
      return "";
    });
  const document = new DOMParser().parseFromString(content, "text/html");
  document.querySelectorAll("script,style").forEach((element) => element.remove());
  document.querySelectorAll("br").forEach((element) => element.replaceWith("\n"));
  const text = document.body.textContent?.trim() ?? "";
  validateCue(startMs, endMs, text, speakerKey);
  return { startMs, endMs, text, speakerKey };
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
  const normalized = source
    .replace(/^\uFEFF/u, "")
    .replace(/\r\n?/gu, "\n")
    .trim();
  const blocks = normalized.split(/\n[\t ]*\n+/u);
  if (blocks[0]?.startsWith("WEBVTT")) blocks.shift();
  const cues = blocks
    .filter((block) => !/^(?:NOTE(?:\s|$)|STYLE(?:\s|$)|REGION(?:\s|$))/u.test(block))
    .map((block) => cue(block, speakerKey));
  if (!cues.length || cues.length > 10_000 || sumBy(cues, (item) => [...item.text].length) > 1_000_000)
    throw new Error("Subtitle script exceeds dubbing limits");
  return cues;
}
