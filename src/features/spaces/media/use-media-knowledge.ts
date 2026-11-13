import { storeToRefs } from "pinia";
import { computed, ref, shallowRef, type Ref } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { admitAiRequirements } from "../../../foundation/ai/ai-readiness-admission";
import { preferredLocalModel } from "../../../foundation/ai/local-model-choice";
import { getErrorMessage } from "../../../foundation/errors/error-format";
import type { LocalModelId } from "../../../shared/contracts/generated/LocalModelId";
import { getCanvasEditorSession } from "../canvas/editor-session";
import { useNodeOperationGate } from "../canvas/use-node-operation-gate";
import { refreshEnrichmentJobs } from "../enrichment/enrichment-jobs-query";
import {
  MEDIA_ENRICHMENT_JOB_TYPES,
  useNodeEnrichmentStatus,
} from "../enrichment/use-node-enrichment-status";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import { isSpaceMediaPayload } from "../model/nodes/capabilities";
import { useSpaceStore } from "../session/space-store";
import {
  editMediaChapter,
  editMediaSummary,
  editTranscriptSegment,
  mediaKnowledgeIsStale,
  shiftMediaChapterStart,
} from "./knowledge-editing";
import type { SpaceTranscriptEdit } from "./schema/knowledge";
import { requestMediaAlignment } from "./transcription";
import { useMediaInsights } from "./use-media-insights";

export function useMediaKnowledge(node: Ref<SpaceCanvasNode>) {
  const store = useSpaceStore();
  const nodeOperations = useNodeOperationGate();
  const { mediaAnalysisIds, mediaAnalysisPreviews, mediaPlayback } = storeToRefs(store);
  const { t } = useI18n();
  const transcriptOpen = ref(false);
  const transcriptionModel = shallowRef<LocalModelId>();
  const alignmentModel = shallowRef<LocalModelId>();
  const alignmentSubmitting = ref(false);
  const submitting = ref(false);
  const { status } = useNodeEnrichmentStatus(
    computed(() => store.activeSpaceId),
    computed(() => node.value.id),
    MEDIA_ENRICHMENT_JOB_TYPES,
  );
  const transcribing = computed(() => submitting.value || Boolean(status.value.transcribing));
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
  const canGenerateInsights = computed(() =>
    Boolean(
      canDerive.value && !analyzing.value && !stale.value && knowledge.value?.transcript.segments.length,
    ),
  );
  const mediaInsights = useMediaInsights(node, () => canGenerateInsights.value, store.generateMediaInsights);
  const insightsDisabledReason = computed(() => {
    if (analyzing.value) return t("spaces.media.analysis.inProgress");
    if (stale.value) return t("spaces.media.analysis.knowledge.stale");
    if (!knowledge.value?.transcript.segments.length) return t("spaces.media.analysis.insights.unavailable");
    return undefined;
  });
  const transcribed = computed(() => Boolean(knowledge.value && !stale.value));
  const canTranscribe = computed(() =>
    Boolean(
      canDerive.value &&
      !analyzing.value &&
      mediaContent.value?.sourceRevision &&
      mediaContent.value.mediaAnalysis?.sourceRevision === mediaContent.value.sourceRevision,
    ),
  );
  async function analyze(chosenModel?: LocalModelId) {
    if (!canTranscribe.value) return;
    submitting.value = true;
    try {
      if (!(await admitAiRequirements([{ kind: "localCapability", capability: "transcription" }]))) return;
      const model = chosenModel ?? transcriptionModel.value ?? (await preferredLocalModel("transcription"));
      if (!model) return;
      await store.analyzeMedia(node.value.id, model);
      await refreshEnrichmentJobs();
    } catch (error) {
      toast.error(getErrorMessage(error));
    } finally {
      submitting.value = false;
    }
  }
  async function alignSegments(segmentIds: string[], chosenModel?: LocalModelId) {
    const current = knowledge.value;
    const spaceId = store.activeSpaceId;
    if (!spaceId || !current || !canGenerateInsights.value || alignmentSubmitting.value) return;
    alignmentSubmitting.value = true;
    try {
      if (!(await admitAiRequirements([{ kind: "localCapability", capability: "speechAlignment" }]))) return;
      const model = chosenModel ?? alignmentModel.value ?? (await preferredLocalModel("speechAlignment"));
      if (!model) return;
      await requestMediaAlignment({
        spaceId,
        nodeId: node.value.id,
        expectedRevision: current.transcript.revision,
        segmentIds,
        model,
      });
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
      expectedContent: data,
      data: { ...data, mediaKnowledge: next },
    });
  }
  async function editKnowledge(
    transform: (current: NonNullable<typeof knowledge.value>) => NonNullable<typeof knowledge.value>,
  ) {
    if (knowledge.value) await saveKnowledge(transform(knowledge.value));
  }
  return {
    transcriptOpen,
    alignmentSubmitting,
    alignSegments,
    transcriptionModel,
    alignmentModel,
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
    transcribing,
    canGenerateInsights,
    insightsDisabledReason,
    analyze,
    seek,
    ...mediaInsights,
    saveSummary: (summary: string) => editKnowledge((k) => editMediaSummary(k, summary)),
    saveChapter: (index: number, title: string, summary: string) =>
      editKnowledge((k) => editMediaChapter(k, index, { title, summary })),
    shiftChapter: (index: number, direction: -1 | 1) =>
      editKnowledge((k) => shiftMediaChapterStart(k, index, direction)),
    saveSegment: (payload: SpaceTranscriptEdit) =>
      editKnowledge((k) => editTranscriptSegment(k, payload.index, payload.text)),
  };
}
