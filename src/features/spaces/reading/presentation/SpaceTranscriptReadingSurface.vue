<script setup lang="ts">
import { Clock } from "@lucide/vue";
import { convertFileSrc } from "@tauri-apps/api/core";
import { refDebounced } from "@vueuse/core";
import { search } from "fast-fuzzy";
import { ToolbarRoot } from "reka-ui";
import { computed, nextTick, ref, shallowRef, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import type { KnowledgeLocator } from "../../knowledge/model/schema/chunk";
import SpaceTextQuote from "../../knowledge/presentation/SpaceTextQuote.vue";
import { formatDurationMs } from "../../media/format-duration";
import SpaceNodeTranscriptionAction from "../../media/presentation/SpaceNodeTranscriptionAction.vue";
import type { SpaceTranscriptSegment } from "../../media/schema/transcript";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceAudioPayload, SpaceVideoPayload } from "../../model/nodes/node-payload";
import SpaceTranscriptPlayer from "./SpaceTranscriptPlayer.vue";

const props = defineProps<{
  node: SpaceCanvasNode<SpaceVideoPayload | SpaceAudioPayload>;
  locator?: KnowledgeLocator | undefined;
  quote?: string | undefined;
}>();

const { t } = useI18n();
const scroller = useTemplateRef<HTMLDivElement>("scroller");
const player = useTemplateRef<InstanceType<typeof SpaceTranscriptPlayer>>("player");
const query = ref("");
const debouncedQuery = refDebounced(query, 120);
const activeSegmentId = shallowRef<string>();

const mediaSrc = computed(() => convertFileSrc(props.node.data.assetPath));
const segments = computed(() => props.node.data.mediaKnowledge?.transcript.segments ?? []);
const chapters = computed(() => props.node.data.mediaKnowledge?.insights?.chapters ?? []);
const visibleSegments = computed(() => {
  const needle = debouncedQuery.value.trim();
  return needle ? search(needle, segments.value, { keySelector: (s) => s.text }) : segments.value;
});

function updateActiveSegment(ms: number) {
  const list = segments.value;
  if (!list.length) return;
  const hit = list.find((s) => ms >= s.startMs && ms < s.endMs);
  const nextId = hit?.id;
  if (nextId !== activeSegmentId.value) activeSegmentId.value = nextId;
}
function isHighlightedSegment(segment: SpaceTranscriptSegment) {
  const loc = props.locator?.location;
  if (loc?.kind === "media") return segment.endMs > loc.startMs && segment.startMs < loc.endMs;
  const needle = props.quote?.trim();
  return Boolean(needle && segment.text.includes(needle));
}
function seekTo(startMs: number) {
  updateActiveSegment(startMs);
  player.value?.seekTo(startMs);
}
function onKeyDown(event: KeyboardEvent) {
  if (event.key !== " ") return;
  const target = event.target as HTMLElement | null;
  if (target?.closest("input, textarea, button, video, audio")) return;
  event.preventDefault();
  player.value?.togglePlayback();
}

watch(
  () => props.node.id,
  () => {
    query.value = "";
    activeSegmentId.value = undefined;
  },
);
watch(
  () => [props.locator, props.quote, segments.value.length] as const,
  async ([locator]) => {
    if (!locator && !props.quote) return;
    if (locator?.location.kind === "media") seekTo(locator.location.startMs);
    await nextTick();
    const matched = visibleSegments.value.find(isHighlightedSegment);
    if (!matched) return;
    scroller.value
      ?.querySelector<HTMLElement>(`[data-segment-start-ms="${matched.startMs}"]`)
      ?.scrollIntoView({ behavior: "smooth", block: "center" });
  },
  { immediate: true },
);
</script>

<template>
  <div class="flex h-full min-h-0 flex-col outline-none" tabindex="-1" @keydown="onKeyDown">
    <SpaceTranscriptPlayer
      ref="player"
      v-model:query="query"
      :kind="node.data.kind"
      :media-src="mediaSrc"
      :chapters="chapters"
      :has-segments="Boolean(segments.length)"
      @seek="seekTo"
      @time-update="updateActiveSegment"
    />
    <div
      v-if="segments.length"
      ref="scroller"
      class="min-h-0 flex-1 divide-y divide-stroke-divider/50 overflow-y-auto bg-surface-raised"
    >
      <div
        v-for="segment in visibleSegments"
        :key="segment.id"
        :data-segment-start-ms="segment.startMs"
        :data-segment-end-ms="segment.endMs"
        class="flex items-start gap-3 px-4 py-3 transition-colors"
        :class="
          activeSegmentId === segment.id ? 'bg-surface-accent-subtle/35' : 'hover:bg-surface-control/35'
        "
      >
        <button
          data-reader-ignore-selection
          type="button"
          class="mt-0.5 inline-flex shrink-0 items-center gap-1 rounded-sm bg-surface-control px-1.5 py-0.5 font-mono text-caption-sm text-fg-secondary tabular-nums select-none hover:bg-surface-control-hover hover:text-fg-primary"
          @click="seekTo(segment.startMs)"
        >
          <Clock class="size-2.5 text-fg-tertiary" />
          <span>{{ formatDurationMs(segment.startMs) }}</span>
        </button>
        <div class="min-w-0 flex-1">
          <SpaceTextQuote :text="segment.text" :exact="isHighlightedSegment(segment) ? quote : undefined" />
        </div>
      </div>
      <p
        v-if="!visibleSegments.length"
        data-reader-ignore-selection
        class="p-4 text-center text-caption text-fg-tertiary select-none"
      >
        {{ t("spaces.transcript.text.noMatches") }}
      </p>
    </div>
    <div
      v-else
      data-reader-ignore-selection
      class="grid min-h-0 flex-1 place-items-center p-6 text-center select-none"
    >
      <div class="max-w-xs space-y-3">
        <p class="text-caption leading-relaxed text-fg-tertiary">
          {{ t("spaces.media.analysis.knowledge.pending") }}
        </p>
        <div class="flex justify-center">
          <ToolbarRoot class="inline-flex rounded-md border border-stroke-divider bg-surface-raised p-0.5">
            <SpaceNodeTranscriptionAction :node="node" />
          </ToolbarRoot>
        </div>
      </div>
    </div>
  </div>
</template>
