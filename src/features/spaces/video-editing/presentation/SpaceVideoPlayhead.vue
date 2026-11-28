<script setup lang="ts">
import { onScopeDispose, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import { timeToTimelineX } from "../timeline/scale";
import type { VideoEditor } from "./use-video-editor";

const props = defineProps<{ editor: VideoEditor }>();
const { t } = useI18n();
const { player, timeline, gestures } = props.editor;
const marker = useTemplateRef<HTMLElement>("marker");
let positionMs = player.positionMs();

/**
 * The playhead is written straight to the element. Routing sixty positions per second through the
 * reactive graph would re-render every lane in the timeline for a one-pixel move.
 */
function place() {
  const element = marker.value;
  if (element) {
    element.style.transform = `translate3d(${timeToTimelineX(positionMs, timeline.pxPerSecond.value)}px, 0, 0)`;
  }
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
    class="pointer-events-none absolute inset-y-0 left-0 z-30 will-change-transform"
    aria-hidden="true"
  >
    <span class="absolute top-[28px] bottom-0 left-0 w-[1.5px] -translate-x-1/2 bg-surface-danger" />
    <span
      class="pointer-events-auto absolute top-[17px] -left-[5.5px] flex h-[14px] w-[11px] cursor-ew-resize touch-none items-start justify-center bg-surface-danger pt-[2.5px] shadow-xs [clip-path:polygon(0_0,100%_0,100%_64%,50%_100%,0_64%)]"
      :title="t('spaces.media.videoEditor.timeline.seek')"
      @pointerdown="gestures.begin($event, { kind: 'scrub' })"
      @pointermove="gestures.move"
      @pointerup="gestures.finish"
      @pointercancel="gestures.cancel"
    >
      <span class="h-[4px] w-px bg-white/85" />
    </span>
  </span>
</template>
