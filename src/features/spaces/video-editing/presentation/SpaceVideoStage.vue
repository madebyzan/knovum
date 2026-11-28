<script setup lang="ts">
import { computed, useTemplateRef } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiDropdownMenu from "../../../../shared/ui/UiDropdownMenu.vue";
import type { VideoEditor } from "./use-video-editor";
import { useVideoStage } from "./use-video-stage";

const props = defineProps<{ editor: VideoEditor }>();
const { t } = useI18n();
const { previewError, converting, selection, retryPreview } = props.editor;
const viewport = useTemplateRef<HTMLElement>("viewport");
const container = useTemplateRef<HTMLDivElement>("container");
const stage = useVideoStage(props.editor, { viewport, container });

const zoomLabel = computed(() =>
  stage.fitted.value
    ? `${t("spaces.media.videoEditor.preview.fit")} · ${Math.round(stage.scale.value * 100)}%`
    : `${Math.round(stage.scale.value * 100)}%`,
);

const zoomItems = computed(() => [
  { id: "fit", label: t("spaces.media.videoEditor.preview.fit"), onSelect: () => stage.fit() },
  { id: "in", label: t("spaces.media.videoEditor.preview.zoomIn"), onSelect: () => stage.zoomBy(1.25) },
  { id: "out", label: t("spaces.media.videoEditor.preview.zoomOut"), onSelect: () => stage.zoomBy(1 / 1.25) },
  { id: "50", label: "50%", onSelect: () => stage.setZoom(0.5) },
  { id: "100", label: "100%", onSelect: () => stage.setZoom(1) },
  { id: "150", label: "150%", onSelect: () => stage.setZoom(1.5) },
]);
</script>

<template>
  <div class="relative flex min-h-0 flex-1 flex-col bg-surface-stage">
    <div
      ref="viewport"
      class="relative grid min-h-0 flex-1 place-items-center overflow-hidden p-3"
      @pointerdown.self="selection.clear()"
    >
      <div
        ref="container"
        class="relative shadow-panel ring-1 ring-white/12"
        :style="stage.frameStyle.value"
        :aria-label="t('spaces.media.videoEditor.preview.stage')"
        role="img"
        @pointerdown.stop
      />
      <div
        v-if="stage.output.value"
        class="pointer-events-none absolute bottom-2.5 left-3 z-10 rounded-md bg-black/55 px-2 py-0.5 font-mono text-caption-sm text-white/75 tabular-nums backdrop-blur-xs"
      >
        {{ stage.output.value.width }} × {{ stage.output.value.height }}
      </div>
      <div class="absolute right-3 bottom-2.5 z-10">
        <UiDropdownMenu :items="zoomItems" align="end">
          <template #trigger>
            <button
              type="button"
              class="rounded-md bg-black/55 px-2 py-0.5 font-mono text-caption-sm text-white/85 tabular-nums backdrop-blur-xs transition-colors hover:bg-black/75 hover:text-white focus-visible:outline-2 focus-visible:outline-stroke-focus"
              :aria-label="t('spaces.media.videoEditor.preview.fit')"
            >
              {{ zoomLabel }}
            </button>
          </template>
        </UiDropdownMenu>
      </div>
    </div>
    <div
      v-if="previewError"
      role="alert"
      class="flex shrink-0 items-center justify-between gap-2 border-t border-stroke-divider bg-surface-panel px-3 py-1.5 text-caption text-fg-danger"
    >
      <span class="truncate">{{ previewError }}</span>
      <UiButton size="sm" @click="retryPreview()">
        {{ t("spaces.media.videoEditor.preview.retry") }}
      </UiButton>
    </div>
    <p
      v-else-if="converting"
      role="status"
      class="shrink-0 truncate border-t border-stroke-divider bg-surface-panel px-3 py-1.5 text-caption text-fg-secondary"
    >
      {{ t("spaces.media.videoEditor.preview.converting") }}
    </p>
  </div>
</template>
