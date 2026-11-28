<script setup lang="ts">
import { LocateFixed } from "@lucide/vue";
import { useVirtualizer } from "@tanstack/vue-virtual";
import { refDebounced } from "@vueuse/core";
import { sumBy } from "es-toolkit";
import { search } from "fast-fuzzy";
import { computed, nextTick, ref, useTemplateRef, watch, type ComponentPublicInstance } from "vue";
import { useI18n } from "vue-i18n";

import type { LocalModelId } from "../../../../shared/contracts/generated/LocalModelId";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiScrollEdgeFade from "../../../../shared/ui/UiScrollEdgeFade.vue";
import UiSearchInput from "../../../../shared/ui/UiSearchInput.vue";
import type { SpaceTranscriptEdit } from "../schema/knowledge";
import type { SpaceMediaTranscript, SpaceTranscriptSegment } from "../schema/transcript";
import { buildSpeakerMap } from "../speaker-display";
import SpaceTranscriptAlignmentControl from "./SpaceTranscriptAlignmentControl.vue";
import SpaceTranscriptSegmentEditor from "./SpaceTranscriptSegmentEditor.vue";

const props = defineProps<{
  nodeId: string;
  segments: SpaceTranscriptSegment[];
  speakerTracks?: readonly SpaceMediaTranscript["speakerTracks"][number][] | undefined;
  transcriptRevision?: number | undefined;
  hasInsights?: boolean;
  currentMs?: number;
  disabled?: boolean;
  alignmentDisabled?: boolean;
}>();
const emit = defineEmits<{
  seek: [positionMs: number];
  save: [payload: SpaceTranscriptEdit];
  align: [segmentIds: string[], model?: LocalModelId];
}>();
const { t } = useI18n();
const scroller = useTemplateRef<HTMLDivElement>("scroller");
const query = ref("");
const debouncedQuery = refDebounced(query, 120);
const autoFollow = ref(true);
const selected = ref<string[]>([]);
const speakers = computed(() =>
  buildSpeakerMap(props.speakerTracks, (index) => t("spaces.transcript.speakers.label", { index })),
);
const canAlign = computed(
  () =>
    selected.value.length > 0 &&
    selected.value.length <= 8 &&
    sumBy(
      props.segments.filter((segment) => selected.value.includes(segment.id)),
      (segment) => segment.endMs - segment.startMs,
    ) <= 300_000,
);
function selectSegment(segmentId: string, checked: boolean) {
  selected.value = checked
    ? [...selected.value, segmentId]
    : selected.value.filter((value) => value !== segmentId);
}
watch(
  () => [props.segments.length, props.transcriptRevision] as const,
  () => {
    selected.value = [];
    virtualizer.value.measure();
  },
);
const visibleIndices = computed(() => {
  const needle = debouncedQuery.value.trim();
  if (!needle) return props.segments.map((_, index) => index);
  const indexed = props.segments.map((segment, index) => ({ segment, index }));
  const matched = search(needle, indexed, {
    keySelector: ({ segment }) => segment.text,
  });
  return matched.map((match) => match.index);
});
const activeIndex = computed(() => {
  const current = props.currentMs;
  if (current === undefined) return -1;
  return props.segments.findLastIndex((segment) => segment.startMs <= current);
});
const activeVisibleIndex = computed(() => visibleIndices.value.indexOf(activeIndex.value));
const virtualizer = useVirtualizer(
  computed(() => ({
    count: visibleIndices.value.length,
    getScrollElement: () => scroller.value,
    getItemKey: (index) => props.segments[visibleIndices.value[index]!]?.id ?? index,
    estimateSize: () => 38,
    overscan: 8,
  })),
);
const rows = computed(() => virtualizer.value.getVirtualItems());
const totalSize = computed(() => virtualizer.value.getTotalSize());
watch(activeVisibleIndex, (index) => {
  if (autoFollow.value && index >= 0) virtualizer.value.scrollToIndex(index, { align: "center" });
});
function measureElement(element: Element | ComponentPublicInstance | null) {
  const row = element instanceof Element ? element : element?.$el;
  if (row instanceof Element) virtualizer.value.measureElement(row);
}
async function remeasure() {
  await nextTick();
  requestAnimationFrame(() => virtualizer.value.measure());
}
</script>

<template>
  <div class="space-y-2">
    <div class="flex items-center gap-1.5">
      <UiSearchInput
        v-model="query"
        size="sm"
        class="min-w-0 flex-1"
        :placeholder="t('spaces.transcript.text.search')"
      />
      <UiIconButton
        :label="t('spaces.media.player.followPlayback')"
        :active="autoFollow"
        tooltip
        size="sm"
        @click="autoFollow = !autoFollow"
      >
        <LocateFixed class="size-3.5" :stroke-width="1.6" />
      </UiIconButton>
    </div>
    <SpaceTranscriptAlignmentControl
      v-if="selected.length > 0"
      :disabled="Boolean(alignmentDisabled)"
      :can-align="canAlign"
      :selected-count="selected.length"
      @align="(model) => emit('align', [...selected], model)"
    />
    <div class="relative -mx-1.5 overflow-hidden">
      <div ref="scroller" class="h-80 overflow-y-auto overscroll-contain pt-0.5 pb-2">
        <div class="relative w-full" :style="{ height: `${totalSize}px` }">
          <div
            v-for="row in rows"
            :key="visibleIndices[row.index]!"
            :ref="measureElement"
            :data-index="row.index"
            class="absolute top-0 left-0 w-full"
            :style="{ transform: `translateY(${row.start}px)` }"
          >
            <SpaceTranscriptSegmentEditor
              :node-id="nodeId"
              :segment="segments[visibleIndices[row.index]!]!"
              :index="visibleIndices[row.index]!"
              :speaker="
                segments[visibleIndices[row.index]!]?.speakerId
                  ? speakers.get(segments[visibleIndices[row.index]!]!.speakerId!)
                  : undefined
              "
              :has-insights="hasInsights"
              :active="visibleIndices[row.index] === activeIndex"
              :selected="selected.includes(segments[visibleIndices[row.index]!]!.id)"
              :selection-active="selected.length > 0"
              :selection-disabled="
                alignmentDisabled ||
                (selected.length >= 8 && !selected.includes(segments[visibleIndices[row.index]!]!.id))
              "
              @select="selectSegment(segments[visibleIndices[row.index]!]!.id, $event)"
              v-bind="disabled === undefined ? {} : { disabled }"
              @seek="emit('seek', $event)"
              @save="emit('save', $event)"
              @editing="remeasure"
            />
          </div>
        </div>
      </div>
      <UiScrollEdgeFade surface="panel" top-class="h-3" bottom-class="h-5" />
    </div>
    <p v-if="!visibleIndices.length" class="py-4 text-center text-caption text-fg-tertiary">
      {{ t("spaces.transcript.text.noMatches") }}
    </p>
  </div>
</template>
