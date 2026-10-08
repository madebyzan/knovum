import { storeToRefs } from "pinia";
import { computed, ref, type Ref } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { getCanvasEditorSession } from "../canvas/editor-session";
import { useNodeOperationGate } from "../canvas/node-operation-gate";
import { refreshEnrichmentJobs } from "../enrichment/enrichment-jobs-query";
import { useNodeEnrichmentStatus } from "../enrichment/use-node-enrichment-status";
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
} from "./knowledge-editing";
import type { SpaceTranscriptEdit } from "./schema/knowledge";
import { requestMediaAlignment } from "./transcription";

export function useMediaKnowledge(node: Ref<SpaceCanvasNode>) {
  const store = useSpaceStore();
  const nodeOperations = useNodeOperationGate();
  const { mediaAnalysisIds, mediaAnalysisPreviews, mediaPlayback } = storeToRefs(store);
  const { t } = useI18n();
  const transcriptOpen = ref(false);
  const alignmentSubmitting = ref(false);
  const submitting = ref(false);
  const { status } = useNodeEnrichmentStatus(
    computed(() => store.activeSpaceId),
    computed(() => node.value.id),
  );
  const analyzing = computed(
    () =>
      submitting.value ||
      status.value.pending > 0 ||
      status.value.running > 0 ||
      status.value.waiting > 0 ||
      mediaAnalysisIds.value.includes(node.value.id),
  );
  const analysisPreview = computed(() => mediaAnalysisPreviews.value[node.value.id]);
  const mediaContent = computed(() => (isSpaceMediaPayload(node.value.data) ? node.value.data : undefined));
  const knowledge = computed(() => mediaContent.value?.mediaKnowledge);
  const insights = computed(() => knowledge.value?.insights);
  const playback = computed(() =>
    mediaPlayback.value?.id === node.value.id ? mediaPlayback.value.state : undefined,
  );
  const stale = computed(() =>
    knowledge.value ? mediaKnowledgeIsStale(knowledge.value, mediaContent.value?.sourceRevision) : false,
  );
  const canEdit = computed(() => nodeOperations.allows("edit"));
  const canDerive = computed(() => nodeOperations.allows("derive"));
  const canGenerateInsights = computed(() => {
    if (!canDerive.value) return false;
    if (analyzing.value) return false;
    if (stale.value) return false;
    if (!knowledge.value) return false;
    if (!knowledge.value.transcript.segments.length) return false;
    return true;
  });
  const insightsDisabledReason = computed(() => {
    if (analyzing.value) return t("spaces.media.analysis.inProgress");
    if (stale.value) return t("spaces.media.analysis.knowledge.stale");
    if (!knowledge.value?.transcript.segments.length) return t("spaces.media.analysis.insights.unavailable");
    return undefined;
  });
  const transcribed = computed(() => Boolean(knowledge.value && !stale.value));
  const canTranscribe = computed(
    () =>
      canDerive.value &&
      !analyzing.value &&
      !transcribed.value &&
      Boolean(
        mediaContent.value?.sourceRevision &&
        mediaContent.value.mediaAnalysis?.sourceRevision === mediaContent.value.sourceRevision,
      ),
  );
  async function analyze() {
    if (!canTranscribe.value) return;
    submitting.value = true;
    try {
      await store.analyzeMedia(node.value.id);
      await refreshEnrichmentJobs();
    } catch (error) {
      toast.error(getErrorMessage(error));
    } finally {
      submitting.value = false;
    }
  }
  async function alignSegments(segmentIds: string[]) {
    const current = knowledge.value;
    const spaceId = store.activeSpaceId;
    if (
      !spaceId ||
      !current ||
      !canDerive.value ||
      stale.value ||
      analyzing.value ||
      alignmentSubmitting.value
    )
      return;
    alignmentSubmitting.value = true;
    try {
      await requestMediaAlignment(spaceId, node.value.id, current.transcript.revision, segmentIds);
      await refreshEnrichmentJobs();
    } catch (error) {
      toast.error(getErrorMessage(error));
    } finally {
      alignmentSubmitting.value = false;
    }
  }
  function seek(positionMs: number) {
    getCanvasEditorSession().reveal([node.value.id], "focus");
    store.requestMediaSeek(node.value.id, positionMs);
  }
  async function generateInsights() {
    if (!canGenerateInsights.value) return;
    try {
      await store.generateMediaInsights(node.value.id, spaceMediaAnalysisLabels());
    } catch (error) {
      toast.error(getErrorMessage(error));
    }
  }
  async function saveKnowledge(next: NonNullable<typeof knowledge.value>) {
    if (!canEdit.value) {
      toast.error(t("spaces.creation.shared.finishNodeOperation"));
      return;
    }
    if (stale.value) throw new Error(t("spaces.media.analysis.knowledge.stale"));
    const data = node.value.data;
    if (!isSpaceMediaPayload(data)) throw new Error(t("spaces.media.analysis.knowledge.stale"));
    await store.saveNodeContent({
      id: node.value.id,
      expectedData: data,
      data: { ...data, mediaKnowledge: next },
    });
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
    alignmentSubmitting,
    alignSegments,
    analyzing,
    analysisPreview,
    knowledge,
    insights,
    playback,
    stale,
    canEdit,
    canDerive,
    canTranscribe,
    transcribed,
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
