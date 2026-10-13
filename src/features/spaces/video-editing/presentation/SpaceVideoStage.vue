<script setup lang="ts">
import { Maximize2, ZoomIn, ZoomOut } from "@lucide/vue";
import { useResizeObserver } from "@vueuse/core";
import { computed, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import SpaceVideoStageSelection from "./SpaceVideoStageSelection.vue";
import type { VideoEditor } from "./use-video-editor";
import { useVideoStageGestures } from "./use-video-stage-gestures";

const props = defineProps<{ editor: VideoEditor }>();
const { t } = useI18n();
const { engine, project, visibleProject, previewError, selection, commit } = props.editor;
const viewport = shallowRef<HTMLElement | null>(null);
const canvas = shallowRef<HTMLCanvasElement | null>(null);
const bounds = shallowRef({ width: 0, height: 0 });
const zoom = shallowRef(1);
const fitted = shallowRef(true);

const output = computed(() => visibleProject.value?.output ?? project.value!.output);
const fitScale = computed(() => {
  if (!bounds.value.width || !bounds.value.height) return 0;
  return Math.min(
    (bounds.value.width - 48) / output.value.width,
    (bounds.value.height - 48) / output.value.height,
  );
});
const scale = computed(() => Math.max(0.02, fitted.value ? fitScale.value : zoom.value));
const frameStyle = computed(() => ({
  width: `${output.value.width * scale.value}px`,
  height: `${output.value.height * scale.value}px`,
}));

const gestures = useVideoStageGestures({
  project: () => project.value,
  selection,
  scale: () => scale.value,
  commit,
});

useResizeObserver(viewport, ([entry]) => {
  if (entry) bounds.value = { width: entry.contentRect.width, height: entry.contentRect.height };
});
watch(canvas, (element) => {
  if (element) engine.attach(element);
});

function setZoom(factor: number) {
  zoom.value = Math.min(4, Math.max(0.05, (fitted.value ? fitScale.value : zoom.value) * factor));
  fitted.value = false;
}
</script>

<template>
  <div class="relative flex min-h-0 flex-1 flex-col bg-surface-muted">
    <div
      :ref="(value) => (viewport = value as HTMLElement | null)"
      class="relative grid min-h-0 flex-1 place-items-center overflow-hidden"
      @pointerdown="selection.clear()"
    >
      <div class="relative shadow-panel" data-video-stage-frame :style="frameStyle" @pointerdown.stop>
        <canvas
          :ref="(value) => (canvas = value as HTMLCanvasElement | null)"
          class="block size-full rounded-sm"
          :aria-label="t('spaces.media.videoEditor.preview.stage')"
        />
        <SpaceVideoStageSelection :editor="editor" :scale="scale" :gestures="gestures" />
      </div>
      <p
        v-if="engine.stalled.value"
        role="status"
        class="pointer-events-none absolute bottom-3 left-1/2 -translate-x-1/2 rounded-full bg-surface-overlay px-3 py-1 text-caption text-fg-secondary shadow-control"
      >
        {{ t("spaces.media.videoEditor.preview.optimizing") }}
      </p>
    </div>
    <div
      v-if="previewError"
      role="alert"
      class="flex shrink-0 items-center justify-between gap-2 border-t border-stroke-divider px-3 py-1.5 text-caption text-fg-danger"
    >
      <span class="truncate">{{ previewError }}</span>
      <UiButton size="sm" @click="previewError = undefined">
        {{ t("spaces.media.videoEditor.preview.retry") }}
      </UiButton>
    </div>
    <footer
      class="flex h-9 shrink-0 items-center justify-end gap-1 border-t border-stroke-divider px-3 text-caption text-fg-tertiary"
    >
      <span class="mr-auto tabular-nums"
        >{{ output.width }} × {{ output.height }} · {{ output.frameRate }} fps</span
      >
      <UiIconButton
        size="sm"
        :label="t('spaces.media.videoEditor.preview.zoomOut')"
        @click="setZoom(1 / 1.25)"
      >
        <ZoomOut class="size-3.5" />
      </UiIconButton>
      <span class="w-10 text-center tabular-nums">{{ Math.round(scale * 100) }}%</span>
      <UiIconButton size="sm" :label="t('spaces.media.videoEditor.preview.zoomIn')" @click="setZoom(1.25)">
        <ZoomIn class="size-3.5" />
      </UiIconButton>
      <UiIconButton
        size="sm"
        tooltip
        :label="t('spaces.media.videoEditor.preview.fit')"
        :disabled="fitted"
        @click="fitted = true"
      >
        <Maximize2 class="size-3.5" />
      </UiIconButton>
    </footer>
  </div>
</template>
