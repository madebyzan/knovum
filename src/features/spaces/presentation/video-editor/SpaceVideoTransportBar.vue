<script setup lang="ts">
import {
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
import { useI18n } from "vue-i18n";

import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import { formatVideoTimelineTime } from "../../video-editing/timecode";

defineProps<{
  playing: boolean;
  currentMs: number;
  durationMs: number;
  canDelete: boolean;
  canUndo: boolean;
  canRedo: boolean;
  zoom: number;
}>();
defineEmits<{
  split: [];
  addText: [];
  remove: [];
  undo: [];
  redo: [];
  toggle: [];
  step: [milliseconds: number];
  zoom: [value: number];
}>();
const { t } = useI18n();
</script>

<template>
  <div
    class="grid h-13 shrink-0 grid-cols-[minmax(0,1fr)_auto_minmax(0,1fr)] items-center gap-3 border-t border-border bg-panel px-3"
  >
    <div class="flex min-w-0 items-center gap-1">
      <UiIconButton :label="t('common.undo')" size="sm" :disabled="!canUndo" @click="$emit('undo')">
        <Undo2 class="size-(--knovum-icon-size-sm)" />
      </UiIconButton>
      <UiIconButton :label="t('common.redo')" size="sm" :disabled="!canRedo" @click="$emit('redo')">
        <Redo2 class="size-(--knovum-icon-size-sm)" />
      </UiIconButton>
      <span class="mx-1 h-5 w-px bg-divider" aria-hidden="true" />
      <UiIconButton :label="t('spaces.media.split')" size="sm" @click="$emit('split')"
        ><Scissors class="size-(--knovum-icon-size-sm)"
      /></UiIconButton>
      <UiIconButton :label="t('spaces.media.addText')" size="sm" @click="$emit('addText')"
        ><Type class="size-(--knovum-icon-size-sm)"
      /></UiIconButton>
      <UiIconButton :label="t('common.delete')" size="sm" :disabled="!canDelete" @click="$emit('remove')"
        ><Trash2 class="size-(--knovum-icon-size-sm)"
      /></UiIconButton>
    </div>
    <div class="flex items-center gap-2">
      <span class="w-12 text-right text-caption font-medium tabular-nums">{{
        formatVideoTimelineTime(currentMs)
      }}</span>
      <UiIconButton :label="t('spaces.media.previousStep')" size="sm" @click="$emit('step', -100)"
        ><SkipBack class="size-(--knovum-icon-size-sm)"
      /></UiIconButton>
      <UiIconButton
        class="size-9 rounded-full bg-foreground text-canvas hover:bg-foreground-secondary hover:text-canvas active:bg-foreground-secondary active:text-canvas"
        :label="playing ? t('spaces.media.pause') : t('spaces.media.play')"
        size="sm"
        @click="$emit('toggle')"
      >
        <Pause v-if="playing" class="size-(--knovum-icon-size-sm)" fill="currentColor" />
        <Play v-else class="size-(--knovum-icon-size-sm)" fill="currentColor" />
      </UiIconButton>
      <UiIconButton :label="t('spaces.media.nextStep')" size="sm" @click="$emit('step', 100)"
        ><SkipForward class="size-(--knovum-icon-size-sm)"
      /></UiIconButton>
      <span class="w-12 text-caption text-foreground-tertiary tabular-nums">{{
        formatVideoTimelineTime(durationMs)
      }}</span>
    </div>
    <div class="flex items-center justify-end gap-1">
      <UiIconButton
        :label="t('spaces.media.zoomOut')"
        size="sm"
        :disabled="zoom <= 1"
        @click="$emit('zoom', Math.max(1, zoom / 1.5))"
      >
        <ZoomOut class="size-(--knovum-icon-size-sm)" />
      </UiIconButton>
      <span class="w-10 text-center text-micro text-foreground-secondary tabular-nums"
        >{{ Math.round(zoom * 100) }}%</span
      >
      <UiIconButton
        :label="t('spaces.media.zoomIn')"
        size="sm"
        :disabled="zoom >= 8"
        @click="$emit('zoom', Math.min(8, zoom * 1.5))"
      >
        <ZoomIn class="size-(--knovum-icon-size-sm)" />
      </UiIconButton>
      <UiIconButton
        :label="t('spaces.media.fitTimeline')"
        size="sm"
        :disabled="zoom === 1"
        @click="$emit('zoom', 1)"
      >
        <Maximize2 class="size-(--knovum-icon-size-sm)" />
      </UiIconButton>
    </div>
  </div>
</template>
