import type { SpaceMediaTranscript, SpaceTranscriptSegment } from "./schema";

const MAX_CHUNK_CHARACTERS = 14_000;

export function mediaTranscriptChunks(transcript: SpaceMediaTranscript) {
  const chunks: string[] = [];
  let current = "";
  for (const segment of transcript.segments) {
    const line = transcriptLine(segment);
    if (current && current.length + line.length + 1 > MAX_CHUNK_CHARACTERS) {
      chunks.push(current);
      current = "";
    }
    current += `${current ? "\n" : ""}${line}`;
  }
  if (current) chunks.push(current);
  return chunks;
}

export function mediaKnowledgeInstruction(pass: "chunk" | "final") {
  const source =
    pass === "final" ? "chapter candidates created from the complete transcript" : "timestamped transcript";
  return [
    `Structure the ${source} into a searchable media knowledge record.`,
    "Return JSON only, with exactly this shape:",
    '{"summary":"...","chapters":[{"startMs":0,"title":"...","summary":"..."}]}',
    "summary must capture the whole supplied material concisely.",
    "chapters must follow content order, use timestamps that exist in the supplied material, and describe topic changes rather than arbitrary time intervals.",
    "The first chapter must start at the first timestamp in the supplied material.",
    "Use the language of the supplied material. Do not add markdown or commentary outside the JSON object.",
  ].join(" ");
}

export function mediaCandidateText(
  records: readonly { summary: string; chapters: readonly MediaChapterCandidate[] }[],
) {
  return records
    .map(
      (record, index) =>
        `Part ${index + 1}\nSummary: ${record.summary}\n${record.chapters
          .map((chapter) => `[${chapter.startMs}ms] ${chapter.title}: ${chapter.summary}`)
          .join("\n")}`,
    )
    .join("\n\n");
}

export interface MediaChapterCandidate {
  startMs: number;
  title: string;
  summary: string;
}

function transcriptLine(segment: SpaceTranscriptSegment) {
  return `[${segment.startMs}ms] ${segment.text}`;
}
