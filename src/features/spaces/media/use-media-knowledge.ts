import { storeToRefs } from "pinia";
import { computed, ref, shallowRef, type Ref } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import type { LocalModelId } from "../../../shared/contracts/generated/LocalModelId";
import { useNodeOperationGate } from "../canvas/use-node-operation-gate";
import {
  MEDIA_ENRICHMENT_JOB_TYPES,
  useNodeEnrichmentStatus,
} from "../enrichment/use-node-enrichment-status";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import { isSpaceMediaPayload } from "../model/nodes/capabilities";
import { buildSourceReaderLocator, classifyReadableSource } from "../reading/source-reader-target";
import { useSpaceStore } from "../session/space-store";
import {
  editMediaChapter,
  editMediaSummary,
  editTranscriptSegment,
  mediaKnowledgeIsStale,
  shiftMediaChapterStart,
} from "./knowledge-editing";
import { runLocalMediaTask } from "./run-local-media-task";
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
  const localLanguage = shallowRef("auto");
  const transcriptionLanguage = computed<string>({
    get: () => store.mediaTranscriptionLanguageByNodeId?.[node.value.id] ?? localLanguage.value,
    set: (value) => {
      localLanguage.value = value;
      if (store.mediaTranscriptionLanguageByNodeId)
        store.mediaTranscriptionLanguageByNodeId[node.value.id] = value;
    },
  });
  const alignmentModel = shallowRef<LocalModelId>();
  const alignmentSubmitting = ref(false);
  const submitting = ref(false);
  const { status, jobs } = useNodeEnrichmentStatus(
    computed(() => store.activeSpaceId),
    computed(() => node.value.id),
    MEDIA_ENRICHMENT_JOB_TYPES,
  );
  const diarizationJob = computed(() => jobs.value.find((job) => job.jobType === "diarize_media_transcript"));
  const transcribing = computed(() => submitting.value || Boolean(status.value.transcribing));
  const analyzing = computed(() =>
    Boolean(
      submitting.value ||
      status.value.pending ||
      status.value.running ||
      status.value.waiting ||
      mediaAnalysisIds.value.includes(node.value.id),
    ),
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
  function analyze(chosenModel?: LocalModelId) {
    if (!canTranscribe.value) return Promise.resolve();
    const lang = transcriptionLanguage.value !== "auto" ? transcriptionLanguage.value : undefined;
    return runLocalMediaTask("transcription", submitting, chosenModel ?? transcriptionModel.value, (model) =>
      lang ? store.analyzeMedia(node.value.id, model, lang) : store.analyzeMedia(node.value.id, model),
    );
  }
  function alignSegments(segmentIds: string[], chosenModel?: LocalModelId) {
    const current = knowledge.value;
    const spaceId = store.activeSpaceId;
    if (!spaceId || !current || !canGenerateInsights.value || alignmentSubmitting.value) {
      return Promise.resolve();
    }
    return runLocalMediaTask(
      "speechAlignment",
      alignmentSubmitting,
      chosenModel ?? alignmentModel.value,
      (model) =>
        requestMediaAlignment({
          spaceId,
          nodeId: node.value.id,
          expectedRevision: current.transcript.revision,
          segmentIds,
          model,
        }),
    );
  }
  function seek(positionMs: number) {
    if (classifyReadableSource(node.value.data) !== "media") return;
    const locator = buildSourceReaderLocator(node.value.data, {
      kind: "media",
      startMs: positionMs,
      endMs: positionMs + 1_000,
    });
    store.openSourceReader({ nodeId: node.value.id, pane: "read", locator });
  }
  async function editKnowledge(
    transform: (current: NonNullable<typeof knowledge.value>) => NonNullable<typeof knowledge.value>,
  ) {
    if (!knowledge.value) return;
    if (!canEdit.value) {
      toast.error(t("spaces.creation.shared.finishNodeOperation"));
      return;
    }
    const data = node.value.data;
    if (stale.value || !isSpaceMediaPayload(data)) {
      throw new Error(t("spaces.media.analysis.knowledge.stale"));
    }
    await store.saveNodeContent({
      id: node.value.id,
      expectedContent: data,
      data: { ...data, mediaKnowledge: transform(knowledge.value) },
    });
  }
  return {
    transcriptOpen,
    alignmentSubmitting,
    alignSegments,
    transcriptionModel,
    transcriptionLanguage,
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
    diarizationJob,
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
