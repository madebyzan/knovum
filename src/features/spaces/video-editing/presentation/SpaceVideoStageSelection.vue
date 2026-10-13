<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import { layerGeometry } from "../geometry";
import type { VideoEditor } from "./use-video-editor";
import type { useVideoStageGestures } from "./use-video-stage-gestures";

const props = defineProps<{
  editor: VideoEditor;
  scale: number;
  gestures: ReturnType<typeof useVideoStageGestures>;
}>();
const { t } = useI18n();
const { project, selection, engine } = props.editor;
const CORNERS = ["nw", "ne", "se", "sw"] as const;

/** Selected clip box in stage pixels, rotated with the layer. */
const clipBox = computed(() => {
  const current = project.value;
  const clip = selection.clip.value;
  const asset = clip && current?.assets.find(({ nodeId }) => nodeId === clip.assetId);
  if (!current || !clip || !asset) return undefined;
  const geometry = layerGeometry(asset, clip.transform, current.output);
  return {
    left: (geometry.centerX - geometry.width / 2) * props.scale,
    top: (geometry.centerY - geometry.height / 2) * props.scale,
    width: geometry.width * props.scale,
    height: geometry.height * props.scale,
    rotation: clip.transform.rotation,
  };
});

const textBox = computed(() => {
  const item = selection.text.value;
  const measured = item ? engine.measureText(item) : undefined;
  if (!measured) return undefined;
  return {
    left: measured.left * props.scale,
    top: measured.top * props.scale,
    width: measured.width * props.scale,
    height: measured.height * props.scale,
  };
});

function cornerClass(corner: (typeof CORNERS)[number]) {
  return {
    nw: "-top-1 -left-1 cursor-nwse-resize",
    ne: "-top-1 -right-1 cursor-nesw-resize",
    se: "-right-1 -bottom-1 cursor-nwse-resize",
    sw: "-bottom-1 -left-1 cursor-nesw-resize",
  }[corner];
}
</script>

<template>
  <div class="pointer-events-none absolute inset-0">
    <div
      v-if="clipBox"
      class="pointer-events-auto absolute cursor-move touch-none outline-2 outline-stroke-accent"
      :style="{
        left: `${clipBox.left}px`,
        top: `${clipBox.top}px`,
        width: `${clipBox.width}px`,
        height: `${clipBox.height}px`,
        transform: `rotate(${clipBox.rotation}deg)`,
      }"
      :aria-label="t('spaces.media.videoEditor.stage.move')"
      @pointerdown="gestures.begin($event, 'move')"
      @pointermove="gestures.move"
      @pointerup="gestures.finish"
      @pointercancel="gestures.cancel"
    >
      <button
        v-for="corner in CORNERS"
        :key="corner"
        type="button"
        class="absolute size-2.5 touch-none rounded-full border border-stroke-on-image bg-surface-accent focus-visible:outline-2 focus-visible:outline-stroke-focus"
        :class="cornerClass(corner)"
        :aria-label="t('spaces.media.videoEditor.stage.resize')"
        @pointerdown="gestures.begin($event, 'scale')"
        @pointermove="gestures.move"
        @pointerup="gestures.finish"
        @pointercancel="gestures.cancel"
      />
      <button
        type="button"
        class="absolute -top-7 left-1/2 size-3 -translate-x-1/2 touch-none rounded-full border border-stroke-on-image bg-surface-accent focus-visible:outline-2 focus-visible:outline-stroke-focus"
        :aria-label="t('spaces.media.videoEditor.stage.rotate')"
        @pointerdown="gestures.begin($event, 'rotate')"
        @pointermove="gestures.move"
        @pointerup="gestures.finish"
        @pointercancel="gestures.cancel"
      />
    </div>
    <div
      v-if="textBox"
      class="pointer-events-auto absolute cursor-move touch-none outline-2 outline-stroke-accent outline-dashed"
      :style="{
        left: `${textBox.left}px`,
        top: `${textBox.top}px`,
        width: `${textBox.width}px`,
        height: `${textBox.height}px`,
      }"
      :aria-label="t('spaces.media.videoEditor.stage.moveText')"
      @pointerdown="gestures.begin($event, 'move')"
      @pointermove="gestures.move"
      @pointerup="gestures.finish"
      @pointercancel="gestures.cancel"
    />
  </div>
</template>
