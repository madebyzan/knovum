export interface SubtitleCue {
  startMs: number;
  endMs: number;
  text: string;
  speaker?: string | undefined;
}

function parseTimestamp(value: string) {
  const match = /^(?:(\d{2,}):)?(\d{2}):(\d{2})[.,](\d{3})$/u.exec(value);
  if (!match || Number(match[2]) >= 60 || Number(match[3]) >= 60) {
    throw new Error("Subtitle timestamp is invalid");
  }
  return (Number(match[1] ?? 0) * 3600 + Number(match[2]) * 60 + Number(match[3])) * 1000 + Number(match[4]);
}

function formatTimestamp(ms: number, decimalSeparator: "," | "."): string {
  const clamped = Math.max(0, Math.round(ms));
  const hours = Math.floor(clamped / 3_600_000);
  const minutes = Math.floor((clamped % 3_600_000) / 60_000);
  const seconds = Math.floor((clamped % 60_000) / 1_000);
  const millis = clamped % 1_000;
  return `${String(hours).padStart(2, "0")}:${String(minutes).padStart(2, "0")}:${String(seconds).padStart(2, "0")}${decimalSeparator}${String(millis).padStart(3, "0")}`;
}

function stripCueMarkup(content: string): string {
  if (typeof DOMParser !== "undefined") {
    const document = new DOMParser().parseFromString(content, "text/html");
    document.querySelectorAll("script,style").forEach((element) => element.remove());
    document.querySelectorAll("br").forEach((element) => element.replaceWith("\n"));
    return document.body.textContent?.trim() ?? "";
  }
  return content
    .replace(/<br\s*\/?>/giu, "\n")
    .replace(/<[^>]+>/gu, "")
    .trim();
}

function parseBlock(block: string): SubtitleCue {
  const lines = block.split("\n");
  const timingIndex = lines[0]?.includes(" --> ") ? 0 : 1;
  const timing = lines[timingIndex]?.match(/^(\S+)\s+-->\s+(\S+)(?:\s+.*)?$/u);
  if (!timing) throw new Error("Subtitle cue has no valid timing");
  const startMs = parseTimestamp(timing[1]!);
  const endMs = parseTimestamp(timing[2]!);
  let speaker: string | undefined;
  const content = lines
    .slice(timingIndex + 1)
    .join("\n")
    .replace(/<v\s+([^>]+)>/gu, (_, found: string) => {
      speaker = found.trim() || undefined;
      return "";
    });
  const text = stripCueMarkup(content);
  if (!text || endMs <= startMs) throw new Error("Subtitle cue exceeds limits");
  return { startMs, endMs, text, ...(speaker ? { speaker } : {}) };
}

/** Parses standard SRT or WebVTT text into validated, timed subtitle cues. */
export function parseSubtitleCues(source: string): SubtitleCue[] {
  const normalized = source
    .replace(/^\uFEFF/u, "")
    .replace(/\r\n?/gu, "\n")
    .trim();
  if (!normalized) return [];
  const blocks = normalized.split(/\n[\t ]*\n+/u);
  if (blocks[0]?.startsWith("WEBVTT")) blocks.shift();
  return blocks
    .filter((block) => !/^(?:NOTE(?:\s|$)|STYLE(?:\s|$)|REGION(?:\s|$))/u.test(block))
    .map(parseBlock);
}

export function formatSubtitleSrt(cues: readonly SubtitleCue[]): string {
  return cues
    .map((cue, index) => {
      const endMs = Math.max(cue.endMs, cue.startMs + 200);
      return `${index + 1}\n${formatTimestamp(cue.startMs, ",")} --> ${formatTimestamp(endMs, ",")}\n${cue.text.trim()}`;
    })
    .join("\n\n");
}

export function formatSubtitleVtt(cues: readonly SubtitleCue[]): string {
  const body = cues
    .map((cue) => {
      const endMs = Math.max(cue.endMs, cue.startMs + 200);
      return `${formatTimestamp(cue.startMs, ".")} --> ${formatTimestamp(endMs, ".")}\n${cue.text.trim()}`;
    })
    .join("\n\n");
  return body ? `WEBVTT\n\n${body}` : "WEBVTT\n";
}
