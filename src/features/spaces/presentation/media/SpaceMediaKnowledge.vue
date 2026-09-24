<script setup lang="ts">
import { AlertTriangle, ChevronDown, ChevronRight, LoaderCircle, ScanText } from "@lucide/vue";
import { computed, toRef } from "vue";
import { useI18n } from "vue-i18n";

import { useMediaKnowledge } from "../../media/use-media-knowledge";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import SpaceNodeEnrichmentStatus from "../SpaceNodeEnrichmentStatus.vue";
import SpaceMediaChapterEditor from "./SpaceMediaChapterEditor.vue";
import SpaceMediaLiveTranscript from "./SpaceMediaLiveTranscript.vue";
import SpaceMediaSummaryEditor from "./SpaceMediaSummaryEditor.vue";
import SpaceTranscriptList from "./SpaceTranscriptList.vue";

const props = defineProps<{ node: SpaceCanvasNode }>();
const { t } = useI18n();
const nodeRef = toRef(props, "node");
const nodeId = computed(() => nodeRef.value.id);

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
  analyze,
  seek,
  generateInsights,
  saveSummary,
  saveChapter,
  shiftChapter,
  saveSegment,
} = useMediaKnowledge(nodeRef);
</script>

<template>
  <section>
    <div class="flex justify-end">
      <button
        type="button"
        class="flex h-(--knovum-control-height-sm) shrink-0 items-center gap-1 rounded-control px-2 text-caption text-foreground-secondary hover:bg-control-hover hover:text-foreground disabled:opacity-55"
        :disabled="analyzing || !canDerive"
        @click="analyze"
      >
        <LoaderCircle v-if="analyzing" class="size-(--knovum-icon-size-xs) animate-spin" />
        <ScanText v-else class="size-(--knovum-icon-size-xs)" />
        {{ t("spaces.media.reanalyze") }}
      </button>
    </div>
    <SpaceNodeEnrichmentStatus :node-id="nodeId" />

    <SpaceMediaLiveTranscript v-if="analysisPreview?.segments.length" :preview="analysisPreview" />

    <template v-if="knowledge">
      <div
        v-if="stale"
        class="mt-3 flex gap-2 rounded-panel border border-warning/35 bg-warning/10 p-2.5 text-caption text-warning"
      >
        <AlertTriangle class="mt-0.5 size-(--knovum-icon-size-sm) shrink-0" />
        <span>{{ t("spaces.media.knowledgeStale") }}</span>
      </div>
      <div v-if="!insights" class="mt-3">
        <button
          type="button"
          class="h-(--knovum-control-height-md) w-full rounded-control bg-accent px-2.5 text-label font-medium text-on-accent transition-colors hover:bg-accent-hover disabled:opacity-55"
          :disabled="!canGenerateInsights"
          :title="insightsDisabledReason"
          @click="generateInsights"
        >
          {{ t("spaces.media.generateInsights") }}
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
        <h4 class="text-micro font-semibold tracking-wide text-foreground-tertiary uppercase">
          {{ t("spaces.transcript.mediaChapters") }}
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
      <div class="mt-4">
        <button
          type="button"
          class="flex h-(--knovum-compact-item-height) w-full items-center gap-2 rounded-control px-2 text-caption font-semibold text-foreground-secondary hover:bg-control-hover hover:text-foreground"
          :aria-expanded="transcriptOpen"
          @click="transcriptOpen = !transcriptOpen"
        >
          <ChevronDown
            v-if="transcriptOpen"
            class="size-(--knovum-icon-size-xs) transition-transform duration-150 ease-standard"
          />
          <ChevronRight
            v-else
            class="size-(--knovum-icon-size-xs) transition-transform duration-150 ease-standard"
          />
          {{ t("spaces.transcript.mediaTranscript") }}
          <span class="ml-auto text-micro font-normal text-foreground-tertiary">
            {{ knowledge.transcript.language }}
            ·
            {{ knowledge.transcript.segments.length }}
          </span>
        </button>
        <SpaceTranscriptList
          v-if="transcriptOpen"
          class="mt-2"
          :node-id="nodeId"
          :segments="knowledge.transcript.segments"
          v-bind="playback ? { currentMs: playback.currentMs } : {}"
          :disabled="stale || !canEdit"
          @seek="seek"
          @save="saveSegment"
        />
      </div>
    </template>
    <p v-else class="mt-3 text-caption leading-relaxed text-foreground-tertiary">
      {{ analyzing ? t("spaces.media.analysisInProgress") : t("spaces.media.knowledgePending") }}
    </p>
  </section>
</template>
