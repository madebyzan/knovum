<script setup lang="ts">
import { Copy, Download, LocateFixed } from "@lucide/vue";
import { refDebounced } from "@vueuse/core";
import { search } from "fast-fuzzy";
import { getActivePinia } from "pinia";
import { ToolbarRoot } from "reka-ui";
import { computed, onBeforeUnmount, onMounted, ref, shallowRef, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiDropdownMenu from "../../../../shared/ui/UiDropdownMenu.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiOutlineRail, { type UiOutlineItem } from "../../../../shared/ui/UiOutlineRail.vue";
import UiScrollEdgeFade from "../../../../shared/ui/UiScrollEdgeFade.vue";
import type { KnowledgeLocator } from "../../knowledge/model/schema/chunk";
import { formatDurationMs } from "../../media/format-duration";
import type { SpaceMediaPlaybackState } from "../../media/playback";
import SpaceNodeTranscriptionAction from "../../media/presentation/SpaceNodeTranscriptionAction.vue";
import type { SpaceTranscriptSegment } from "../../media/schema/transcript";
import { buildSpeakerMap } from "../../media/speaker-display";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceAudioPayload, SpaceVideoPayload } from "../../model/nodes/node-payload";
import { useSpaceStore } from "../../session/space-store";
import SpaceTranscriptCue from "./SpaceTranscriptCue.vue";
import SpaceTranscriptPlayer from "./SpaceTranscriptPlayer.vue";
import { useTranscriptExport } from "./use-transcript-export";
import { useTranscriptVirtualizer } from "./use-transcript-virtualizer";

const props = withDefaults(
  defineProps<{
    node: SpaceCanvasNode<SpaceVideoPayload | SpaceAudioPayload>;
    locator?: KnowledgeLocator | undefined;
    locateNonce?: number | undefined;
    quote?: string | undefined;
    visible?: boolean;
  }>(),
  { visible: true },
);

const { t } = useI18n();
const store = getActivePinia() ? useSpaceStore() : undefined;
const scroller = useTemplateRef<HTMLDivElement>("scroller");
const player = useTemplateRef<InstanceType<typeof SpaceTranscriptPlayer>>("player");
const query = ref("");
const debouncedQuery = refDebounced(query, 120);
const autoFollow = shallowRef(true);
const activeSegmentId = shallowRef<string>();
const currentMs = shallowRef(0);

const transcript = computed(() => props.node.data.mediaKnowledge?.transcript);
const segments = computed(() => transcript.value?.segments ?? []);
const speakers = computed(() =>
  buildSpeakerMap(transcript.value?.speakerTracks, (index) =>
    t("spaces.transcript.speakers.label", { index }),
  ),
);
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
const { rows, paddingTop, paddingBottom, measureElement, scrollToSegment, resetScroll, refreshLayout } =
  useTranscriptVirtualizer(scroller, visibleSegments);

function updateActiveSegment(ms: number) {
  currentMs.value = ms;
  const hit = segments.value.findLast((s) => ms >= s.startMs) ?? segments.value[0];
  if (hit && hit.id !== activeSegmentId.value) {
    activeSegmentId.value = hit.id;
    if (autoFollow.value && !debouncedQuery.value.trim()) scrollToSegment(hit.startMs, "center");
  }
}
function handlePlayback(state: SpaceMediaPlaybackState) {
  store?.updateMediaPlayback(props.node.id, state);
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
const { copyTranscript, exportItems } = useTranscriptExport(() => props.node.data.title, segments, speakers);
const KEY_SKIP_MS: Record<string, number> = { ArrowLeft: -5_000, ArrowRight: 5_000 };
function isFindShortcut(event: KeyboardEvent) {
  return (event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "f";
}
function onKeyDown(event: KeyboardEvent) {
  if (isFindShortcut(event)) {
    event.preventDefault();
    player.value?.focusSearch();
    return;
  }
  if ((event.target as HTMLElement | null)?.closest("input, textarea, button, video, audio")) return;
  const skipDelta = KEY_SKIP_MS[event.key];
  if (skipDelta) {
    event.preventDefault();
    player.value?.skipBy(skipDelta);
  } else if (event.key === " ") {
    event.preventDefault();
    void player.value?.togglePlayback();
  }
}
function locateCurrent() {
  if (props.locator?.location.kind === "media") seekTo(props.locator.location.startMs, true);
  if (props.locator || props.quote) scrollToHighlighted();
}
function resetReadingState() {
  query.value = "";
  activeSegmentId.value = undefined;
  currentMs.value = 0;
  resetScroll();
}

onMounted(() => {
  refreshLayout();
  locateCurrent();
});
onBeforeUnmount(() => store?.clearMediaPlayback(props.node.id));
watch(() => props.node.id, resetReadingState);
watch(debouncedQuery, resetScroll);
watch(
  () => props.visible,
  (shown) => {
    if (shown) refreshLayout();
  },
  { flush: "post" },
);
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
      :visible="visible"
      @time-update="updateActiveSegment"
      @playback="handlePlayback"
    >
      <template #actions>
        <UiIconButton
          size="sm"
          tooltip
          :active="autoFollow"
          :label="t('spaces.media.player.followPlayback')"
          @click="autoFollow = !autoFollow"
        >
          <LocateFixed class="size-3.5" :stroke-width="1.6" />
        </UiIconButton>
        <UiIconButton size="sm" tooltip :label="t('spaces.transcript.text.copyAll')" @click="copyTranscript">
          <Copy class="size-3.5" />
        </UiIconButton>
        <UiDropdownMenu :items="exportItems" align="end">
          <template #trigger>
            <UiIconButton size="sm" tooltip :label="t('spaces.transcript.text.export')">
              <Download class="size-3.5" />
            </UiIconButton>
          </template>
        </UiDropdownMenu>
      </template>
    </SpaceTranscriptPlayer>
    <div v-if="segments.length" class="relative flex min-h-0 flex-1 flex-col overflow-hidden">
      <div
        ref="scroller"
        class="transcript-rows min-h-0 flex-1 overflow-x-hidden overflow-y-auto bg-surface-raised py-2 pb-10"
        :class="chapterOutline.length ? 'pr-9 pl-3' : 'px-3'"
      >
        <div
          v-if="visibleSegments.length"
          class="w-full"
          :style="{ paddingTop: `${paddingTop}px`, paddingBottom: `${paddingBottom}px` }"
        >
          <div
            v-for="row in rows"
            :key="String(row.key)"
            :ref="measureElement"
            :data-index="row.index"
            class="w-full pb-0.5"
          >
            <SpaceTranscriptCue
              v-if="visibleSegments[row.index]"
              :segment="visibleSegments[row.index]!"
              :speaker="
                visibleSegments[row.index]!.speakerId
                  ? speakers.get(visibleSegments[row.index]!.speakerId!)
                  : undefined
              "
              :active="activeSegmentId === visibleSegments[row.index]!.id"
              :current-ms="activeSegmentId === visibleSegments[row.index]!.id ? currentMs : undefined"
              :highlighted="isHighlightedSegment(visibleSegments[row.index]!)"
              :quote="quote"
              @seek="seekTo"
            />
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
