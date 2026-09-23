<script setup lang="ts">
import { Captions, Film, Volume2 } from "@lucide/vue";
import { computed } from "vue";

import { formatDurationMs } from "../../media/format-duration";
import { clipDuration, projectDuration, type VideoEditProject } from "../../video-editing/project";

const props = defineProps<{
  project: VideoEditProject;
  playheadMs: number;
  selectedClipId?: string | undefined;
  selectedTextId?: string | undefined;
}>();
const emit = defineEmits<{
  seek: [milliseconds: number];
  selectClip: [id: string];
  selectText: [id: string];
}>();
const duration = computed(() => Math.max(projectDuration(props.project), 1));
const ticks = computed(() => Array.from({ length: 11 }, (_, index) => (duration.value * index) / 10));
const clipStyle = (milliseconds: number) => ({ width: `${(milliseconds / duration.value) * 100}%` });
const rangeStyle = (startMs: number, endMs: number) => ({
  left: `${(startMs / duration.value) * 100}%`,
  width: `${((endMs - startMs) / duration.value) * 100}%`,
});

function seek(event: MouseEvent) {
  const bounds = (event.currentTarget as HTMLElement).getBoundingClientRect();
  emit("seek", ((event.clientX - bounds.left) / bounds.width) * duration.value);
}
</script>

<template>
  <section class="relative h-52 shrink-0 border-t border-border bg-panel">
    <div class="grid h-7 grid-cols-[7rem_1fr] border-b border-divider text-micro text-foreground-tertiary">
      <div class="border-r border-divider" />
      <button
        type="button"
        class="relative cursor-default"
        :aria-label="$t('spaces.media.seekTimeline')"
        @click="seek"
      >
        <span
          v-for="tick in ticks"
          :key="tick"
          class="absolute top-0 h-full border-l border-divider"
          :style="{ left: `${(tick / duration) * 100}%` }"
        >
          <span class="ml-1 tabular-nums">{{ formatDurationMs(tick) }}</span>
        </span>
      </button>
    </div>
    <div class="relative grid h-16 grid-cols-[7rem_1fr] border-b border-divider">
      <div class="flex items-center gap-2 border-r border-divider px-3 text-caption">
        <Film class="size-(--knovum-icon-size-sm)" /><span>{{ $t("spaces.media.videoTrack") }}</span
        ><Volume2 class="ml-auto size-(--knovum-icon-size-xs)" />
      </div>
      <div class="relative flex items-center gap-px overflow-hidden px-1">
        <button
          v-for="clip in project.videoTrack.clips"
          :key="clip.id"
          type="button"
          class="h-10 min-w-2 overflow-hidden rounded-grid-cell border bg-accent-subtle px-2 text-left text-caption text-accent"
          :class="selectedClipId === clip.id ? 'border-accent ring-1 ring-accent' : 'border-accent/25'"
          :style="clipStyle(clipDuration(clip))"
          @click="$emit('selectClip', clip.id)"
        >
          <span class="block truncate"
            >{{ formatDurationMs(clip.startMs) }} – {{ formatDurationMs(clip.endMs) }}</span
          >
        </button>
      </div>
    </div>
    <div class="grid h-16 grid-cols-[7rem_1fr] border-b border-divider">
      <div class="flex items-center gap-2 border-r border-divider px-3 text-caption">
        <Captions class="size-(--knovum-icon-size-sm)" /><span>{{ $t("spaces.media.textTrack") }}</span>
      </div>
      <div class="relative m-1">
        <button
          v-for="item in project.textTrack.items"
          :key="item.id"
          type="button"
          class="absolute top-2 h-9 overflow-hidden rounded-grid-cell border bg-selected px-2 text-left text-caption text-accent"
          :class="selectedTextId === item.id ? 'border-accent ring-1 ring-accent' : 'border-accent/20'"
          :style="rangeStyle(item.startMs, item.endMs)"
          @click="$emit('selectText', item.id)"
        >
          {{ item.text }}
        </button>
      </div>
    </div>
    <span
      class="pointer-events-none absolute top-7 bottom-0 z-10 w-px bg-danger"
      :style="{ left: `calc(7rem + (100% - 7rem) * ${playheadMs / duration})` }"
    />
  </section>
</template>
