<script setup lang="ts">
import { Maximize2, ZoomIn, ZoomOut } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import type { VideoEditor } from "./use-video-editor";
import { useVideoStage } from "./use-video-stage";

const props = defineProps<{ editor: VideoEditor }>();
const { t } = useI18n();
const { previewError, selection } = props.editor;
const stage = useVideoStage(props.editor);
</script>

<template>
  <div class="relative flex min-h-0 flex-1 flex-col bg-surface-stage">
    <div
      :ref="(value) => (stage.viewport.value = value as HTMLElement | null)"
      class="relative grid min-h-0 flex-1 place-items-center overflow-hidden"
      @pointerdown.self="selection.clear()"
    >
      <div
        :ref="(value) => (stage.container.value = value as HTMLDivElement | null)"
        class="relative shadow-panel ring-1 ring-white/10"
        :style="stage.frameStyle.value"
        :aria-label="t('spaces.media.videoEditor.preview.stage')"
        role="img"
        @pointerdown.stop
      />
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
      class="flex h-9 shrink-0 items-center justify-end gap-1 border-t border-stroke-divider bg-surface-panel px-3 text-caption text-fg-tertiary"
    >
      <span v-if="stage.output.value" class="mr-auto tabular-nums">
        {{ stage.output.value.width }} × {{ stage.output.value.height }} ·
        {{ stage.output.value.frameRate }} fps
      </span>
      <UiIconButton
        size="sm"
        :label="t('spaces.media.videoEditor.preview.zoomOut')"
        @click="stage.zoomBy(1 / 1.25)"
      >
        <ZoomOut class="size-3.5" />
      </UiIconButton>
      <span class="w-10 text-center tabular-nums">{{ Math.round(stage.scale.value * 100) }}%</span>
      <UiIconButton
        size="sm"
        :label="t('spaces.media.videoEditor.preview.zoomIn')"
        @click="stage.zoomBy(1.25)"
      >
        <ZoomIn class="size-3.5" />
      </UiIconButton>
      <UiIconButton
        size="sm"
        tooltip
        :label="t('spaces.media.videoEditor.preview.fit')"
        :disabled="stage.fitted.value"
        @click="stage.fit()"
      >
        <Maximize2 class="size-3.5" />
      </UiIconButton>
    </footer>
  </div>
</template>
