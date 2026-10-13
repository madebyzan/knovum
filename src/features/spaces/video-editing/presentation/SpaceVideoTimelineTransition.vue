<script setup lang="ts">
import { Blend } from "@lucide/vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import type { VideoTransition } from "../project";
import { timeToPx } from "../timeline/scale";
import { clipEndMs } from "../timing";
import type { VideoEditor } from "./use-video-editor";

const props = defineProps<{ transition: VideoTransition; editor: VideoEditor }>();
const { t } = useI18n();
const { timeline, gestures, selection, visibleProject } = props.editor;

const geometry = computed(() => {
  const project = visibleProject.value;
  const clip = project?.videoTrack.clips.find(({ id }) => id === props.transition.fromClipId);
  if (!clip) return undefined;
  const width = Math.max(14, timeToPx(props.transition.durationMs, timeline.pxPerSecond.value));
  const center = timeToPx(clipEndMs(clip) - props.transition.durationMs / 2, timeline.pxPerSecond.value);
  return { left: `${center - width / 2}px`, width: `${width}px` };
});
const selected = computed(() => selection.selection.value?.id === props.transition.id);
</script>

<template>
  <button
    v-if="geometry"
    type="button"
    class="absolute inset-y-2 z-10 grid touch-none place-items-center rounded border bg-surface-overlay/90 text-fg-secondary shadow-control hover:text-fg-primary focus-visible:outline-2 focus-visible:outline-stroke-focus"
    :class="selected ? 'border-stroke-accent text-fg-accent' : 'border-stroke-default'"
    :style="geometry"
    :aria-label="`${t(`spaces.media.videoEditor.transitions.kinds.${transition.kind}`)} · ${transition.durationMs} ms`"
    @pointerdown="gestures.begin($event, { kind: 'transition', id: transition.id })"
    @pointermove="gestures.move"
    @pointerup="gestures.finish"
    @pointercancel="gestures.cancel"
  >
    <Blend class="size-3" />
  </button>
</template>
