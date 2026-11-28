<script setup lang="ts">
import { Magnet, Maximize2, ZoomIn, ZoomOut } from "@lucide/vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiSlider from "../../../../shared/ui/UiSlider.vue";
import { MAX_PX_PER_SECOND } from "../timeline/scale";
import type { VideoEditor } from "./use-video-editor";

const MIN_PX_PER_SECOND = 2;
const LOG_MIN = Math.log(MIN_PX_PER_SECOND);
const LOG_RANGE = Math.log(MAX_PX_PER_SECOND) - LOG_MIN;

const props = defineProps<{ editor: VideoEditor }>();
const { t } = useI18n();
const { timeline, gestures } = props.editor;
const zoomSlider = computed({
  get: () => (Math.log(Math.max(MIN_PX_PER_SECOND, timeline.pxPerSecond.value)) - LOG_MIN) / LOG_RANGE,
  set: (normalized: number) => timeline.setPxPerSecond(Math.exp(LOG_MIN + normalized * LOG_RANGE)),
});
</script>

<template>
  <div class="flex items-center justify-end gap-1">
    <UiIconButton
      :label="t('spaces.media.videoEditor.timeline.snapping')"
      :active="gestures.snapEnabled.value"
      tooltip
      size="sm"
      @click="gestures.toggleSnap()"
    >
      <Magnet class="size-3.5" />
    </UiIconButton>
    <span class="mx-1 h-5 w-px bg-stroke-divider" aria-hidden="true" />
    <UiIconButton
      :label="t('spaces.media.videoEditor.timeline.zoomOut')"
      tooltip
      size="sm"
      @click="timeline.zoom(1 / 1.4)"
    >
      <ZoomOut class="size-3.5" />
    </UiIconButton>
    <UiSlider
      v-model="zoomSlider"
      class="w-20"
      :min="0"
      :max="1"
      :step="0.01"
      :label="t('spaces.media.videoEditor.timeline.zoomIn')"
    />
    <UiIconButton
      :label="t('spaces.media.videoEditor.timeline.zoomIn')"
      tooltip
      size="sm"
      @click="timeline.zoom(1.4)"
    >
      <ZoomIn class="size-3.5" />
    </UiIconButton>
    <UiIconButton :label="t('spaces.media.videoEditor.timeline.fit')" tooltip size="sm" @click="timeline.fit">
      <Maximize2 class="size-3.5" />
    </UiIconButton>
  </div>
</template>
