<script setup lang="ts">
import type { VideoAudioClip } from "../project";
import { formatVideoEditTime } from "../timecode";
const props = defineProps<{ clip: VideoAudioClip; title: string; durationMs: number; selected: boolean }>();
defineEmits<{ select: [] }>();
</script>
<template>
  <div class="relative h-8 border-b border-stroke-divider">
    <button
      type="button"
      class="absolute inset-y-1 min-w-0 truncate rounded-sm border bg-surface-selected px-2 text-left text-caption-sm text-fg-accent focus-visible:outline-2 focus-visible:outline-stroke-focus"
      :class="selected ? 'border-stroke-accent ring-1 ring-stroke-accent' : 'border-stroke-accent/30'"
      :style="{
        left: `${(clip.timelineStartMs / durationMs) * 100}%`,
        width: `${((clip.endMs - clip.startMs) / durationMs) * 100}%`,
      }"
      :aria-label="`${title} · ${formatVideoEditTime(clip.timelineStartMs)}`"
      @click="$emit('select')"
    >
      {{ title }}
    </button>
  </div>
</template>
