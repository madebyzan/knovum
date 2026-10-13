<script setup lang="ts">
import {
  CopyPlus,
  Maximize2,
  Pause,
  Play,
  Redo2,
  Scissors,
  SkipBack,
  SkipForward,
  Trash2,
  Type,
  Undo2,
  ZoomIn,
  ZoomOut,
} from "@lucide/vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import { formatVideoEditTime } from "../timecode";
import { projectDurationMs } from "../timing";
import type { VideoEditor } from "./use-video-editor";

const props = defineProps<{ editor: VideoEditor }>();
const { t } = useI18n();
const { engine, project, history, actions, timeline, selection } = props.editor;
const duration = computed(() => (project.value ? projectDurationMs(project.value) : 0));
</script>

<template>
  <div
    class="grid h-11 shrink-0 grid-cols-[minmax(0,1fr)_auto_minmax(0,1fr)] items-center gap-3 border-b border-stroke-divider bg-surface-panel px-3"
  >
    <div class="flex min-w-0 items-center gap-1">
      <UiIconButton
        :label="t('common.undo')"
        size="sm"
        :disabled="!history.canUndo.value"
        @click="history.undo"
      >
        <Undo2 class="size-3.5" />
      </UiIconButton>
      <UiIconButton
        :label="t('common.redo')"
        size="sm"
        :disabled="!history.canRedo.value"
        @click="history.redo"
      >
        <Redo2 class="size-3.5" />
      </UiIconButton>
      <span class="mx-1 h-5 w-px bg-stroke-divider" aria-hidden="true" />
      <UiIconButton
        :label="t('spaces.media.videoEditor.clip.split')"
        tooltip
        size="sm"
        :disabled="!actions.canSplit.value"
        @click="actions.split"
      >
        <Scissors class="size-3.5" />
      </UiIconButton>
      <UiIconButton
        :label="t('spaces.media.videoEditor.clip.duplicate')"
        tooltip
        size="sm"
        :disabled="!selection.clip.value"
        @click="actions.duplicate"
      >
        <CopyPlus class="size-3.5" />
      </UiIconButton>
      <UiIconButton
        :label="t('spaces.media.videoEditor.text.add')"
        tooltip
        size="sm"
        @click="actions.addText()"
      >
        <Type class="size-3.5" />
      </UiIconButton>
      <UiIconButton
        :label="t('common.delete')"
        tooltip
        size="sm"
        :disabled="!actions.canDelete.value"
        @click="actions.remove"
      >
        <Trash2 class="size-3.5" />
      </UiIconButton>
    </div>
    <div class="flex items-center gap-1.5">
      <span class="w-22 text-right text-caption font-medium tabular-nums">{{
        formatVideoEditTime(engine.displayMs.value)
      }}</span>
      <UiIconButton
        :label="t('spaces.media.videoEditor.playback.goToStart')"
        size="sm"
        @click="actions.jump('start')"
      >
        <SkipBack class="size-3.5" />
      </UiIconButton>
      <UiIconButton
        class="size-8 rounded-full bg-surface-inverse text-fg-inverse hover:bg-surface-inverse-hover hover:text-fg-inverse"
        :label="engine.playing.value ? t('common.pause') : t('spaces.media.player.play')"
        size="sm"
        @click="engine.toggle"
      >
        <Pause v-if="engine.playing.value" class="size-3.5" fill="currentColor" />
        <Play v-else class="size-3.5" fill="currentColor" />
      </UiIconButton>
      <UiIconButton
        :label="t('spaces.media.videoEditor.playback.goToEnd')"
        size="sm"
        @click="actions.jump('end')"
      >
        <SkipForward class="size-3.5" />
      </UiIconButton>
      <span class="w-22 text-caption text-fg-tertiary tabular-nums">{{ formatVideoEditTime(duration) }}</span>
    </div>
    <div class="flex items-center justify-end gap-1">
      <UiIconButton
        :label="t('spaces.media.videoEditor.playback.stepBackward')"
        tooltip
        size="sm"
        @click="actions.step(-1)"
      >
        <SkipBack class="size-3" />
      </UiIconButton>
      <UiIconButton
        :label="t('spaces.media.videoEditor.playback.stepForward')"
        tooltip
        size="sm"
        @click="actions.step(1)"
      >
        <SkipForward class="size-3" />
      </UiIconButton>
      <span class="mx-1 h-5 w-px bg-stroke-divider" aria-hidden="true" />
      <UiIconButton
        :label="t('spaces.media.videoEditor.timeline.zoomOut')"
        size="sm"
        @click="timeline.zoom(1 / 1.4)"
      >
        <ZoomOut class="size-3.5" />
      </UiIconButton>
      <UiIconButton
        :label="t('spaces.media.videoEditor.timeline.zoomIn')"
        size="sm"
        @click="timeline.zoom(1.4)"
      >
        <ZoomIn class="size-3.5" />
      </UiIconButton>
      <UiIconButton
        :label="t('spaces.media.videoEditor.timeline.fit')"
        tooltip
        size="sm"
        @click="timeline.fit"
      >
        <Maximize2 class="size-3.5" />
      </UiIconButton>
    </div>
  </div>
</template>
