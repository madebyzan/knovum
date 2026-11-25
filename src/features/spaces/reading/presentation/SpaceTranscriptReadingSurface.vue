<script setup lang="ts">
import { refDebounced } from "@vueuse/core";
import { search } from "fast-fuzzy";
import { ToolbarRoot } from "reka-ui";
import { computed, onMounted, ref, shallowRef, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiOutlineRail, { type UiOutlineItem } from "../../../../shared/ui/UiOutlineRail.vue";
import UiScrollEdgeFade from "../../../../shared/ui/UiScrollEdgeFade.vue";
import type { KnowledgeLocator } from "../../knowledge/model/schema/chunk";
import SpaceTextQuote from "../../knowledge/presentation/SpaceTextQuote.vue";
import { formatDurationMs } from "../../media/format-duration";
import SpaceNodeTranscriptionAction from "../../media/presentation/SpaceNodeTranscriptionAction.vue";
import type { SpaceTranscriptSegment } from "../../media/schema/transcript";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceAudioPayload, SpaceVideoPayload } from "../../model/nodes/node-payload";
import SpaceTranscriptPlayer from "./SpaceTranscriptPlayer.vue";
import { useTranscriptVirtualizer } from "./use-transcript-virtualizer";

const props = defineProps<{
  node: SpaceCanvasNode<SpaceVideoPayload | SpaceAudioPayload>;
  locator?: KnowledgeLocator | undefined;
  locateNonce?: number | undefined;
  quote?: string | undefined;
}>();

const { t } = useI18n();
const scroller = useTemplateRef<HTMLDivElement>("scroller");
const player = useTemplateRef<InstanceType<typeof SpaceTranscriptPlayer>>("player");
const query = ref("");
const debouncedQuery = refDebounced(query, 120);
const activeSegmentId = shallowRef<string>();
const currentMs = shallowRef(0);

const segments = computed(() => props.node.data.mediaKnowledge?.transcript.segments ?? []);
const chapters = computed(() => props.node.data.mediaKnowledge?.insights?.chapters ?? []);
const chapterOutline = computed<readonly UiOutlineItem[]>(() =>
  chapters.value.map((c) => ({
    id: String(c.startMs),
    label: c.title,
    meta: formatDurationMs(c.startMs),
    level: 1,
  })),
);
const activeChapterId = computed(() => {
  const hit = chapters.value.findLast((c) => currentMs.value >= c.startMs) ?? chapters.value[0];
  return hit ? String(hit.startMs) : undefined;
});
const visibleSegments = computed(() => {
  const needle = debouncedQuery.value.trim();
  return needle ? search(needle, segments.value, { keySelector: (s) => s.text }) : segments.value;
});
const { rows, totalSize, measureElement, scrollToSegment } = useTranscriptVirtualizer(
  scroller,
  visibleSegments,
);

function updateActiveSegment(ms: number) {
  currentMs.value = ms;
  const hit = segments.value.find((s) => ms >= s.startMs && ms < s.endMs);
  if (hit?.id !== activeSegmentId.value) activeSegmentId.value = hit?.id;
}
function isHighlightedSegment(segment: SpaceTranscriptSegment) {
  const loc = props.locator?.location;
  if (loc?.kind === "media") return segment.endMs > loc.startMs && segment.startMs < loc.endMs;
  const needle = props.quote?.trim();
  return Boolean(needle && segment.text.includes(needle));
}
function scrollToHighlighted() {
  const matched = visibleSegments.value.find(isHighlightedSegment);
  if (matched) scrollToSegment(matched.startMs, "center");
}
function segmentRowClass(segment: SpaceTranscriptSegment) {
  if (isHighlightedSegment(segment)) return "";
  if (activeSegmentId.value === segment.id) return "bg-surface-control/75";
  return "hover:bg-surface-control-hover/55";
}
function seekTo(startMs: number, autoplay = true) {
  updateActiveSegment(startMs);
  void player.value?.seekTo(startMs, autoplay);
}
function jumpToChapter(id: string) {
  const startMs = Number(id);
  if (!Number.isFinite(startMs)) return;
  seekTo(startMs);
  const target = visibleSegments.value.find((s) => s.endMs > startMs);
  if (target) scrollToSegment(target.startMs, "start");
}
function onKeyDown(event: KeyboardEvent) {
  if (
    event.key !== " " ||
    (event.target as HTMLElement | null)?.closest("input, textarea, button, video, audio")
  )
    return;
  event.preventDefault();
  player.value?.togglePlayback();
}
function locateCurrent() {
  if (props.locator?.location.kind === "media") seekTo(props.locator.location.startMs, true);
  if (props.locator || props.quote) scrollToHighlighted();
}
function resetReadingState() {
  query.value = "";
  activeSegmentId.value = undefined;
  currentMs.value = 0;
  if (scroller.value) scroller.value.scrollTop = 0;
}

onMounted(locateCurrent);
watch(() => props.node.id, resetReadingState);
watch(() => [props.locator, props.locateNonce] as const, locateCurrent, { flush: "post" });
watch(
  () => [props.quote, segments.value.length] as const,
  () => (props.locator || props.quote) && scrollToHighlighted(),
  { flush: "post" },
);
</script>

<template>
  <div class="flex h-full min-h-0 flex-col outline-none" tabindex="-1" @keydown="onKeyDown">
    <SpaceTranscriptPlayer
      ref="player"
      v-model:query="query"
      :media="node.data"
      :has-segments="Boolean(segments.length)"
      @time-update="updateActiveSegment"
    />
    <div v-if="segments.length" class="relative flex min-h-0 flex-1 overflow-hidden">
      <div
        ref="scroller"
        class="transcript-rows min-h-0 flex-1 overflow-y-auto bg-surface-raised py-2 pb-10"
        :class="chapterOutline.length ? 'pr-10 pl-3' : 'px-3'"
      >
        <div v-if="visibleSegments.length" class="relative w-full" :style="{ height: `${totalSize}px` }">
          <div
            v-for="row in rows"
            :key="visibleSegments[row.index]!.id"
            :ref="measureElement"
            :data-index="row.index"
            class="absolute top-0 left-0 w-full pb-0.5"
            :style="{ transform: `translateY(${row.start}px)` }"
          >
            <div
              :data-segment-start-ms="visibleSegments[row.index]!.startMs"
              :data-segment-end-ms="visibleSegments[row.index]!.endMs"
              :data-cited-segment="isHighlightedSegment(visibleSegments[row.index]!) ? '' : undefined"
              :data-active-segment="activeSegmentId === visibleSegments[row.index]!.id ? '' : undefined"
              class="relative flex items-baseline gap-2.5 rounded-lg px-2.5 py-2 transition-colors duration-120"
              :class="segmentRowClass(visibleSegments[row.index]!)"
            >
              <button
                data-reader-ignore-selection
                type="button"
                class="w-11 shrink-0 text-left font-mono text-caption-sm tabular-nums transition-colors select-none hover:text-fg-primary focus-visible:outline-2 focus-visible:outline-stroke-focus"
                :class="
                  activeSegmentId === visibleSegments[row.index]!.id
                    ? 'font-medium text-fg-accent'
                    : 'text-fg-tertiary'
                "
                @click="seekTo(visibleSegments[row.index]!.startMs)"
              >
                {{ formatDurationMs(visibleSegments[row.index]!.startMs) }}
              </button>
              <div class="min-w-0 flex-1 text-body leading-relaxed text-fg-primary">
                <SpaceTextQuote
                  :text="visibleSegments[row.index]!.text"
                  :exact="isHighlightedSegment(visibleSegments[row.index]!) ? quote : undefined"
                />
              </div>
            </div>
          </div>
        </div>
        <p
          v-else
          data-reader-ignore-selection
          class="p-4 text-center text-caption text-fg-tertiary select-none"
        >
          {{ t("spaces.transcript.text.noMatches") }}
        </p>
      </div>
      <UiScrollEdgeFade surface="raised" top-class="h-4" bottom-class="h-9" />
      <UiOutlineRail
        v-if="chapterOutline.length"
        :items="chapterOutline"
        :active-id="activeChapterId"
        :label="t('spaces.transcript.chapters.title')"
        @select="jumpToChapter"
      />
    </div>
    <div
      v-else
      data-reader-ignore-selection
      class="flex min-h-0 flex-1 flex-col items-center justify-center gap-3 p-6 text-center select-none"
    >
      <p class="max-w-xs text-caption leading-relaxed text-fg-tertiary">
        {{ t("spaces.media.analysis.knowledge.pending") }}
      </p>
      <ToolbarRoot class="inline-flex rounded-md border border-stroke-divider bg-surface-raised p-0.5">
        <SpaceNodeTranscriptionAction :node="node" />
      </ToolbarRoot>
    </div>
  </div>
</template>

<style src="./reader-citation-highlight.css"></style>
