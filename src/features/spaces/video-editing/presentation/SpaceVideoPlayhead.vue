<script setup lang="ts">
import { onScopeDispose, useTemplateRef, watch } from "vue";

import { timeToPx } from "../timeline/scale";
import type { VideoEditor } from "./use-video-editor";

const props = defineProps<{ editor: VideoEditor }>();
const { player, timeline } = props.editor;
const marker = useTemplateRef<HTMLElement>("marker");
let positionMs = player.positionMs();

/**
 * The playhead is written straight to the element. Routing sixty positions per second through the
 * reactive graph would re-render every lane in the timeline for a one-pixel move.
 */
function place() {
  const element = marker.value;
  if (element)
    element.style.transform = `translate3d(${timeToPx(positionMs, timeline.pxPerSecond.value)}px, 0, 0)`;
}

onScopeDispose(
  player.subscribe((timeMs) => {
    positionMs = timeMs;
    place();
  }),
);
watch([marker, timeline.pxPerSecond], place);
</script>

<template>
  <span
    ref="marker"
    class="pointer-events-none absolute inset-y-0 z-30 w-px bg-surface-danger will-change-transform"
    aria-hidden="true"
  >
    <span class="absolute top-0 -left-[3px] size-[7px] rounded-b-sm bg-surface-danger" />
  </span>
</template>
