<script setup lang="ts">
import { AlertTriangle, ChevronRight } from "@lucide/vue";
import { CollapsibleContent, CollapsibleRoot, CollapsibleTrigger } from "reka-ui";
import { computed, toRef } from "vue";
import { useI18n } from "vue-i18n";

import { useMediaKnowledge } from "../../media/use-media-knowledge";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import SpaceNodeEnrichmentStatus from "../enrichment/SpaceNodeEnrichmentStatus.vue";
import SpaceDiarizationControl from "./SpaceDiarizationControl.vue";
import SpaceMediaChapterEditor from "./SpaceMediaChapterEditor.vue";
import SpaceMediaLiveTranscript from "./SpaceMediaLiveTranscript.vue";
import SpaceMediaSummaryEditor from "./SpaceMediaSummaryEditor.vue";
import SpaceTranscriptNavigator from "./SpaceTranscriptNavigator.vue";

const props = defineProps<{ node: SpaceCanvasNode }>();
const { t } = useI18n();
const currentNode = toRef(props, "node");
const nodeId = computed(() => currentNode.value.id);

const {
  transcriptOpen,
  analyzing,
  analysisPreview,
  knowledge,
  insights,
  playback,
  stale,
  canEdit,
  canDerive,
  canGenerateInsights,
  insightsDisabledReason,
  seek,
  generateInsights,
  saveSummary,
  saveChapter,
  shiftChapter,
  saveSegment,
  alignSegments,
  alignmentSubmitting,
} = useMediaKnowledge(currentNode);
</script>

<template>
  <section>
    <SpaceNodeEnrichmentStatus :node-id="nodeId" />

    <SpaceMediaLiveTranscript v-if="analysisPreview?.segments.length" :preview="analysisPreview" />

    <template v-if="knowledge">
      <div
        v-if="stale"
        class="mt-3 flex gap-2 rounded-panel border border-stroke-warning/35 bg-surface-warning-subtle p-2.5 text-caption text-fg-warning"
      >
        <AlertTriangle class="mt-0.5 size-3.5 shrink-0" />
        <span>{{ t("spaces.media.analysis.knowledge.stale") }}</span>
      </div>
      <div v-if="!insights" class="mt-3">
        <button
          type="button"
          class="h-(--knovum-control-height-md) w-full rounded-md bg-surface-accent px-2.5 text-label font-medium text-fg-on-accent transition-colors hover:bg-surface-accent-hover disabled:opacity-40"
          :disabled="!canGenerateInsights"
          :title="insightsDisabledReason"
          @click="generateInsights"
        >
          {{ t("spaces.media.analysis.insights.generate") }}
        </button>
      </div>
      <SpaceMediaSummaryEditor
        v-if="insights"
        class="mt-3"
        :node-id="nodeId"
        :summary="insights.summary"
        :disabled="stale || !canEdit"
        @save="saveSummary"
      />
      <div v-if="insights" class="mt-4">
        <h4 class="text-caption-sm font-semibold tracking-wide text-fg-tertiary uppercase">
          {{ t("spaces.transcript.chapters.title") }}
        </h4>
        <div class="mt-1 space-y-1">
          <SpaceMediaChapterEditor
            v-for="(chapter, index) in insights.chapters"
            :key="chapter.startMs"
            :node-id="nodeId"
            :chapter="chapter"
            :index="index"
            :active="
              !!playback && playback.currentMs >= chapter.startMs && playback.currentMs < chapter.endMs
            "
            :disabled="stale || !canEdit"
            @seek="seek"
            @save="saveChapter"
            @shift="shiftChapter"
          />
        </div>
      </div>
      <CollapsibleRoot v-model:open="transcriptOpen" as-child>
        <div class="mt-4">
          <CollapsibleTrigger
            class="flex w-full items-center gap-2 rounded-md px-2 py-2 text-caption font-semibold text-fg-secondary hover:bg-surface-control-hover hover:text-fg-primary"
          >
            <ChevronRight
              class="size-3 transition-transform duration-150 ease-standard"
              :class="transcriptOpen && 'rotate-90'"
            />
            {{ t("spaces.transcript.text.title") }}
            <span class="ml-auto text-caption-sm font-normal text-fg-tertiary">
              {{ knowledge.transcript.language }}
              ·
              {{ knowledge.transcript.segments.length }}
            </span>
          </CollapsibleTrigger>
          <CollapsibleContent class="mt-2">
            <SpaceDiarizationControl
              :node-id="nodeId"
              :revision="knowledge.transcript.revision"
              :duration-ms="knowledge.transcript.durationMs"
              :speaker-count="knowledge.transcript.speakerTracks.length"
              :disabled="stale || !canDerive || analyzing || alignmentSubmitting"
            />
            <SpaceTranscriptNavigator
              :node-id="nodeId"
              :segments="knowledge.transcript.segments"
              v-bind="playback ? { currentMs: playback.currentMs } : {}"
              :disabled="stale || !canEdit || analyzing || alignmentSubmitting"
              :alignment-disabled="stale || !canDerive || analyzing || alignmentSubmitting"
              @seek="seek"
              @save="saveSegment"
              @align="alignSegments"
            />
          </CollapsibleContent>
        </div>
      </CollapsibleRoot>
    </template>
    <p v-else class="mt-3 text-caption leading-relaxed text-fg-tertiary">
      {{ analyzing ? t("spaces.media.analysis.inProgress") : t("spaces.media.analysis.knowledge.pending") }}
    </p>
  </section>
</template>
