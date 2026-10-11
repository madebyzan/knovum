<script setup lang="ts">
import { defaultRangeExtractor, useVirtualizer } from "@tanstack/vue-virtual";
import { useResizeObserver } from "@vueuse/core";
import {
  computed,
  inject,
  onMounted,
  onUpdated,
  shallowRef,
  useTemplateRef,
  watch,
  type ComponentPublicInstance,
} from "vue";

import { overlayScopeVisibleKey } from "../../../../shared/contracts/overlay-scope-visibility";
import type { VideoLineDraft } from "../video-dubbing-draft";
import { SPEECH_EDITOR_VIEWPORT_KEY } from "./speech-editor-viewport";
import VideoDubbingLineEditor from "./VideoDubbingLineEditor.vue";

const props = defineProps<{
  lines: readonly VideoLineDraft[];
  voiceOptions: readonly { value: string; label: string }[];
  deliveryInstructionVoiceIds: ReadonlySet<string>;
  lexiconValid: boolean;
  showSource: boolean;
  disabled: boolean;
}>();
const emit = defineEmits<{
  update: [index: number, line: VideoLineDraft];
  remove: [index: number];
  preview: [text: string, revisionId: string, deliveryInstruction?: string];
  source: [startMs: number, endMs: number];
}>();
const viewportRef = inject(SPEECH_EDITOR_VIEWPORT_KEY);
if (!viewportRef) throw new Error("Video dubbing requires the project editor viewport");
const linesRef = useTemplateRef<HTMLElement>("linesRef");
const scrollMargin = shallowRef(0);
const focusedLineIndex = shallowRef<number>();
function updateScrollMargin() {
  const element = linesRef.value;
  const scroll = viewportRef!.value;
  if (element && scroll)
    scrollMargin.value =
      element.getBoundingClientRect().top - scroll.getBoundingClientRect().top + scroll.scrollTop;
}
onMounted(updateScrollMargin);
onUpdated(updateScrollMargin);
useResizeObserver(viewportRef, updateScrollMargin);
const virtualizer = useVirtualizer(
  computed(() => ({
    count: props.lines.length,
    getScrollElement: () => viewportRef.value,
    scrollMargin: scrollMargin.value,
    rangeExtractor: (range) =>
      [
        ...new Set([
          ...defaultRangeExtractor(range),
          ...(focusedLineIndex.value === undefined ? [] : [focusedLineIndex.value]),
        ]),
      ].toSorted((a, b) => a - b),
    getItemKey: (index: number) => props.lines[index]!.key,
    estimateSize: () => 260,
    overscan: 4,
  })),
);
const visibility = inject(overlayScopeVisibleKey);
watch(
  () => visibility?.value,
  (shown) => {
    if (shown) updateScrollMargin();
  },
  { flush: "post" },
);
const rows = computed(() => virtualizer.value.getVirtualItems());
const totalHeight = computed(() => virtualizer.value.getTotalSize());
function measureLine(element: Element | ComponentPublicInstance | null) {
  const row = element instanceof Element ? element : element?.$el;
  if (row instanceof Element) virtualizer.value.measureElement(row);
}
</script>

<template>
  <div
    ref="linesRef"
    class="relative"
    :style="{ height: `${totalHeight}px` }"
    @focusin="
      focusedLineIndex = Number(
        ($event.target as HTMLElement).closest('[data-index]')?.getAttribute('data-index'),
      )
    "
    @focusout="focusedLineIndex = undefined"
  >
    <div
      v-for="row in rows"
      :key="String(row.key)"
      :ref="measureLine"
      :data-index="row.index"
      class="absolute top-0 left-0 w-full pb-2"
      :style="{ transform: `translateY(${row.start - scrollMargin}px)` }"
    >
      <VideoDubbingLineEditor
        :model-value="lines[row.index]!"
        :voice-options
        :delivery-instruction-voice-ids
        :lexicon-valid
        :show-source
        :disabled
        @update:model-value="emit('update', row.index, $event)"
        @remove="emit('remove', row.index)"
        @preview="(text, revision, instruction) => emit('preview', text, revision, instruction)"
        @source="(start, end) => emit('source', start, end)"
      />
    </div>
  </div>
</template>
