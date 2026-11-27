<script setup lang="ts">
import { AlertTriangle, Sparkles } from "@lucide/vue";
import { computed, toRef } from "vue";
import { useI18n } from "vue-i18n";

import ModelChoiceTrigger from "../../../../foundation/ai/ModelChoiceTrigger.vue";
import {
  useLocalModelOptions,
  useRoutedTargetOptions,
} from "../../../../foundation/ai/use-model-choice-options";
import UiButton from "../../../../shared/ui/UiButton.vue";
import SpaceNodeEnrichmentStatus from "../../enrichment/presentation/SpaceNodeEnrichmentStatus.vue";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { useMediaKnowledge } from "../use-media-knowledge";
import SpaceMediaChapterEditor from "./SpaceMediaChapterEditor.vue";
import SpaceMediaLiveTranscript from "./SpaceMediaLiveTranscript.vue";
import SpaceMediaSummaryEditor from "./SpaceMediaSummaryEditor.vue";
import SpaceMediaTranscriptCollapsible from "./SpaceMediaTranscriptCollapsible.vue";
import SpaceNodeTranscriptionAction from "./SpaceNodeTranscriptionAction.vue";
import SpaceTranscriptionLanguageSelect from "./SpaceTranscriptionLanguageSelect.vue";

const props = defineProps<{ node: SpaceCanvasNode }>();
const { t } = useI18n();
const currentNode = toRef(props, "node");
const nodeId = computed(() => currentNode.value.id);
const media = useMediaKnowledge(currentNode);
const {
  options: knowledgeTargets,
  defaultTarget: defaultKnowledgeTarget,
  targetForId: knowledgeTargetForId,
} = useRoutedTargetOptions(() => "knowledge");
const { options: transcriptionModels } = useLocalModelOptions(() => "transcription");
const editLocked = computed(() => media.stale.value || !media.canEdit.value);
const busy = computed(() => media.stale.value || media.analyzing.value || media.alignmentSubmitting.value);
const deriveLocked = computed(() => busy.value || !media.canDerive.value);
const transcriptEditLocked = computed(() => busy.value || !media.canEdit.value);
const transcribeLocked = computed(() => !media.canTranscribe.value || media.alignmentSubmitting.value);

function isChapterActive(startMs: number, endMs: number) {
  const current = media.playback.value?.currentMs;
  return current !== undefined && current >= startMs && current < endMs;
}

function runInsights(targetId?: string) {
  const target = targetId ? knowledgeTargetForId(targetId) : defaultKnowledgeTarget.value;
  void media.generateInsights(target);
}
</script>

<template>
  <section>
    <SpaceNodeEnrichmentStatus :node-id="nodeId" />
    <SpaceMediaLiveTranscript
      v-if="media.analysisPreview.value?.segments.length"
      :preview="media.analysisPreview.value"
    />
    <template v-if="media.knowledge.value">
      <div
        v-if="media.stale.value"
        class="mt-2.5 flex gap-2 rounded-lg border border-stroke-warning/35 bg-surface-warning-subtle px-2.5 py-2 text-caption text-fg-warning"
      >
        <AlertTriangle class="mt-0.5 size-3.5 shrink-0" />
        <span>{{ t("spaces.media.analysis.knowledge.stale") }}</span>
      </div>
      <div v-if="!media.insights.value" class="mt-2.5">
        <ModelChoiceTrigger
          :options="knowledgeTargets"
          :disabled="!media.canGenerateInsights.value"
          @select="runInsights"
        >
          <template #trigger="{ multiple }">
            <UiButton
              variant="secondary"
              size="sm"
              class="w-full border border-stroke-divider/90 bg-surface-raised/90 text-fg-primary shadow-control hover:border-stroke-default-hover hover:bg-surface-raised"
              :disabled="!media.canGenerateInsights.value"
              :title="media.insightsDisabledReason.value"
              @click="!multiple && runInsights()"
            >
              <Sparkles class="size-3.5 text-fg-tertiary" :stroke-width="1.6" />
              {{ t("spaces.media.analysis.insights.generate") }}
            </UiButton>
          </template>
        </ModelChoiceTrigger>
      </div>
      <SpaceMediaSummaryEditor
        v-if="media.insights.value"
        class="mt-2.5"
        :node-id="nodeId"
        :summary="media.insights.value.summary"
        :disabled="editLocked"
        :can-regenerate="media.canGenerateInsights.value"
        :regenerate-disabled-reason="media.insightsDisabledReason.value"
        :regenerate-options="knowledgeTargets"
        @save="media.saveSummary"
        @regenerate="runInsights"
      />
      <div v-if="media.insights.value" class="mt-3.5">
        <h4 class="text-caption font-medium text-fg-tertiary">
          {{ t("spaces.transcript.chapters.title") }}
        </h4>
        <div class="mt-1 space-y-1">
          <SpaceMediaChapterEditor
            v-for="(chapter, index) in media.insights.value.chapters"
            :key="chapter.startMs"
            :node-id="nodeId"
            :chapter="chapter"
            :index="index"
            :active="isChapterActive(chapter.startMs, chapter.endMs)"
            :disabled="editLocked"
            @seek="media.seek"
            @save="media.saveChapter"
            @shift="media.shiftChapter"
          />
        </div>
      </div>
      <SpaceMediaTranscriptCollapsible
        v-model:open="media.transcriptOpen.value"
        v-model:language="media.transcriptionLanguage.value"
        :node-id="nodeId"
        :transcript="media.knowledge.value.transcript"
        :transcription-models="transcriptionModels"
        :diarization-job="media.diarizationJob.value"
        :current-ms="media.playback.value?.currentMs"
        :transcribe-locked="transcribeLocked"
        :derive-locked="deriveLocked"
        :transcript-edit-locked="transcriptEditLocked"
        @analyze="media.analyze"
        @seek="media.seek"
        @save-segment="media.saveSegment"
        @align-segments="media.alignSegments"
      />
    </template>
    <div v-else class="mt-2 space-y-2">
      <p class="text-caption leading-relaxed text-fg-tertiary">
        {{
          media.analyzing.value
            ? t("spaces.media.analysis.inProgress")
            : t("spaces.media.analysis.knowledge.pending")
        }}
      </p>
      <template v-if="media.canTranscribe.value">
        <SpaceTranscriptionLanguageSelect
          v-model="media.transcriptionLanguage.value"
          :disabled="transcribeLocked"
        />
        <SpaceNodeTranscriptionAction :node="node" />
      </template>
    </div>
  </section>
</template>
