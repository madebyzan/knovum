<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import type { VideoTextOverlay } from "../project";
import { timeToPx } from "../timeline/scale";
import type { VideoEditor } from "./use-video-editor";

const props = defineProps<{ item: VideoTextOverlay; editor: VideoEditor }>();
const { t } = useI18n();
const { timeline, gestures, selection } = props.editor;
const EDGES = ["start", "end"] as const;

const selected = computed(() => selection.selection.value?.id === props.item.id);
const geometry = computed(() => ({
  left: `${timeToPx(props.item.startMs, timeline.pxPerSecond.value)}px`,
  width: `${Math.max(10, timeToPx(props.item.endMs - props.item.startMs, timeline.pxPerSecond.value))}px`,
}));

function grab(event: PointerEvent) {
  gestures.begin(event, {
    kind: "text-move",
    id: props.item.id,
    grabMs: timeline.timeAt(event) - props.item.startMs,
  });
}
</script>

<template>
  <div
    class="absolute inset-y-1 overflow-hidden rounded border bg-surface-selected"
    :class="selected ? 'border-stroke-accent ring-1 ring-stroke-accent' : 'border-stroke-accent/25'"
    :style="geometry"
  >
    <button
      type="button"
      class="size-full cursor-grab touch-none truncate px-2 text-left text-caption-sm text-fg-accent focus-visible:outline-2 focus-visible:outline-stroke-focus active:cursor-grabbing"
      :aria-label="item.text"
      @pointerdown="grab"
      @pointermove="gestures.move"
      @pointerup="gestures.finish"
      @pointercancel="gestures.cancel"
    >
      {{ item.text }}
    </button>
    <button
      v-for="edge in EDGES"
      :key="edge"
      type="button"
      class="absolute inset-y-0 w-1.5 cursor-ew-resize touch-none bg-surface-accent/40 hover:bg-surface-accent focus-visible:outline-2 focus-visible:outline-stroke-focus"
      :class="edge === 'start' ? 'left-0' : 'right-0'"
      :aria-label="
        t(
          edge === 'start'
            ? 'spaces.media.videoEditor.trim.textStart'
            : 'spaces.media.videoEditor.trim.textEnd',
        )
      "
      @pointerdown="gestures.begin($event, { kind: 'text-trim', id: item.id, edge })"
      @pointermove="gestures.move"
      @pointerup="gestures.finish"
      @pointercancel="gestures.cancel"
    />
  </div>
</template>
