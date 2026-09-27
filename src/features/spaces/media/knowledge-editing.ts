import { normalizeChapterEnds } from "./chapters";
import type { SpaceMediaChapter, SpaceMediaInsights, SpaceMediaKnowledge } from "./knowledge-schema";

export function mediaKnowledgeIsStale(knowledge: SpaceMediaKnowledge, sourceRevision?: string) {
  return !sourceRevision || knowledge.sourceRevision !== sourceRevision;
}

export function editMediaSummary(knowledge: SpaceMediaKnowledge, summary: string) {
  return editInsights(knowledge, { summary: requireText(summary, "Media summary") });
}

export function editMediaChapter(
  knowledge: SpaceMediaKnowledge,
  index: number,
  patch: Pick<SpaceMediaChapter, "title" | "summary">,
) {
  const insights = requireInsights(knowledge);
  const chapters = insights.chapters.map((chapter, chapterIndex) =>
    chapterIndex === index
      ? Object.assign({}, chapter, {
          title: requireText(patch.title, "Chapter title"),
          summary: requireText(patch.summary, "Chapter summary"),
        })
      : chapter,
  );
  return editInsights(knowledge, { chapters });
}

export function shiftMediaChapterStart(knowledge: SpaceMediaKnowledge, index: number, direction: -1 | 1) {
  const insights = requireInsights(knowledge);
  if (index <= 0 || index >= insights.chapters.length) return knowledge;
  const starts = knowledge.transcript.segments.map((segment) => segment.startMs);
  const chapter = insights.chapters[index]!;
  const current = starts.indexOf(chapter.startMs);
  if (current < 0) throw new Error("Chapter start is not anchored to the transcript");
  const nextStart = starts[current + direction];
  const lower = insights.chapters[index - 1]!.startMs;
  const upper = insights.chapters[index + 1]?.startMs ?? knowledge.transcript.durationMs;
  if (nextStart === undefined || nextStart <= lower || nextStart >= upper) return knowledge;
  const chapters = insights.chapters.map((candidate) => Object.assign({}, candidate));
  chapters[index]!.startMs = nextStart;
  return editInsights(knowledge, {
    chapters: normalizeChapterEnds(chapters, knowledge.transcript.durationMs),
  });
}

export function editTranscriptSegment(knowledge: SpaceMediaKnowledge, index: number, text: string) {
  const segments = knowledge.transcript.segments.map((segment) => ({ ...segment }));
  const segment = segments[index];
  if (!segment) throw new Error("Transcript segment does not exist");
  segment.text = requireText(text, "Transcript segment");
  const transcript = {
    ...knowledge.transcript,
    segments,
    text: segments.map(({ text: value }) => value).join(" "),
  };
  return {
    ...knowledge,
    transcript,
    insights: editedInsights(knowledge.insights),
  };
}

function editInsights(
  knowledge: SpaceMediaKnowledge,
  patch: Partial<SpaceMediaInsights>,
): SpaceMediaKnowledge {
  const insights = requireInsights(knowledge);
  return { ...knowledge, insights: { ...insights, ...patch, editedAt: Date.now() } };
}
function editedInsights(insights?: SpaceMediaInsights) {
  return insights ? { ...insights, editedAt: Date.now() } : undefined;
}
function requireInsights(knowledge: SpaceMediaKnowledge) {
  if (!knowledge.insights) throw new Error("Media insights have not been generated");
  return knowledge.insights;
}
function requireText(value: string, label: string) {
  const text = value.trim();
  if (!text) throw new Error(`${label} cannot be empty`);
  return text;
}
