import { storeToRefs } from "pinia";
import { computed, ref, type Ref } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { useCanvasEditorSession } from "../canvas/editor-session";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import { isSpaceMediaPayload } from "../model/nodes/capabilities";
import { useSpaceStore } from "../session/space-store";
import { spaceMediaAnalysisLabels } from "./analysis-labels";
import {
  editMediaChapter,
  editMediaSummary,
  editTranscriptSegment,
  mediaKnowledgeIsStale,
  shiftMediaChapterStart,
} from "./editing";
import type { SpaceTranscriptEdit } from "./schema";

export function useMediaKnowledge(node: Ref<SpaceCanvasNode>) {
  const store = useSpaceStore();
  const { mediaAnalysisIds, mediaAnalysisPreviews, mediaPlayback } = storeToRefs(store);
  const { t } = useI18n();
  const transcriptOpen = ref(false);
  const analyzing = computed(() => mediaAnalysisIds.value.includes(node.value.id));
  const analysisPreview = computed(() => mediaAnalysisPreviews.value[node.value.id]);
  const mediaData = computed(() => (isSpaceMediaPayload(node.value.data) ? node.value.data : undefined));
  const knowledge = computed(() => mediaData.value?.mediaKnowledge);
  const insights = computed(() => knowledge.value?.insights);
  const playback = computed(() =>
    mediaPlayback.value?.id === node.value.id ? mediaPlayback.value.state : undefined,
  );
  const stale = computed(() =>
    knowledge.value ? mediaKnowledgeIsStale(knowledge.value, mediaData.value?.sourceRevision) : false,
  );
  const canGenerateInsights = computed(() => {
    if (analyzing.value) return false;
    if (stale.value) return false;
    if (!knowledge.value) return false;
    if (!knowledge.value.transcript.segments.length) return false;
    return true;
  });
  const insightsDisabledReason = computed(() => {
    if (analyzing.value) return t("spaces.media.analysisInProgress");
    if (stale.value) return t("spaces.media.knowledgeStale");
    if (!knowledge.value?.transcript.segments.length) return t("spaces.media.insightsUnavailable");
    return undefined;
  });
  async function analyze() {
    try {
      await store.analyzeMedia(node.value.id, spaceMediaAnalysisLabels());
    } catch (error) {
      toast.error(String(error));
    }
  }
  function seek(positionMs: number) {
    useCanvasEditorSession().reveal([node.value.id], "focus");
    store.requestMediaSeek(node.value.id, positionMs);
  }
  async function generateInsights() {
    if (!canGenerateInsights.value) return;
    try {
      await store.generateMediaInsights(node.value.id, spaceMediaAnalysisLabels());
    } catch (error) {
      toast.error(String(error));
    }
  }
  async function saveKnowledge(next: NonNullable<typeof knowledge.value>) {
    if (stale.value) throw new Error(t("spaces.media.knowledgeStale"));
    const data = node.value.data;
    if (!isSpaceMediaPayload(data)) throw new Error(t("spaces.media.knowledgeStale"));
    await store.saveNodeData(node.value.id, { ...data, mediaKnowledge: next });
  }
  async function saveSummary(summary: string) {
    if (knowledge.value) await saveKnowledge(editMediaSummary(knowledge.value, summary));
  }
  async function saveChapter(index: number, title: string, summary: string) {
    if (knowledge.value) await saveKnowledge(editMediaChapter(knowledge.value, index, { title, summary }));
  }
  async function shiftChapter(index: number, direction: -1 | 1) {
    if (knowledge.value) await saveKnowledge(shiftMediaChapterStart(knowledge.value, index, direction));
  }
  async function saveSegment(payload: SpaceTranscriptEdit) {
    if (knowledge.value)
      await saveKnowledge(editTranscriptSegment(knowledge.value, payload.index, payload.text));
  }
  return {
    transcriptOpen,
    analyzing,
    analysisPreview,
    knowledge,
    insights,
    playback,
    stale,
    canGenerateInsights,
    insightsDisabledReason,
    analyze,
    seek,
    generateInsights,
    saveSummary,
    saveChapter,
    shiftChapter,
    saveSegment,
  };
}
