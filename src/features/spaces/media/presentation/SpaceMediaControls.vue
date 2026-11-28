<script setup lang="ts">
import { Pause, Play, Volume2, VolumeX } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import { formatDurationMs } from "../format-duration";
import SpaceMediaTimeline from "./SpaceMediaTimeline.vue";

withDefaults(
  defineProps<{
    playing: boolean;
    currentMs: number;
    durationMs: number;
    rate?: number;
    muted?: boolean;
    density?: "compact" | "regular";
  }>(),
  { density: "regular", rate: 1, muted: false },
);
const emit = defineEmits<{
  toggle: [];
  seek: [positionMs: number];
  cycleRate: [];
  toggleMute: [];
}>();
const { t } = useI18n();
</script>

<template>
  <div
    class="nodrag pointer-events-auto flex items-center rounded-full border border-white/15 bg-black/75 text-white shadow-dialog backdrop-blur-md select-none"
    :class="density === 'compact' ? 'h-7 gap-1.5 px-2' : 'h-8 gap-2 px-2.5'"
  >
    <button
      type="button"
      class="grid shrink-0 place-items-center rounded-full text-white/90 transition-colors duration-100 hover:bg-white/20 hover:text-white focus-visible:ring-2 focus-visible:ring-white/50 focus-visible:outline-none active:bg-white/30"
      :class="density === 'compact' ? 'size-5' : 'size-6'"
      :aria-label="playing ? t('common.pause') : t('spaces.media.player.play')"
      @click="emit('toggle')"
    >
      <Pause v-if="playing" class="size-3" fill="currentColor" />
      <Play v-else class="ml-0.5 size-3" fill="currentColor" />
    </button>
    <SpaceMediaTimeline :current-ms="currentMs" :duration-ms="durationMs" @seek="emit('seek', $event)" />
    <span class="shrink-0 text-caption-sm font-medium text-white/80 tabular-nums">
      {{ formatDurationMs(currentMs) }} / {{ formatDurationMs(durationMs) }}
    </span>
    <button
      type="button"
      class="shrink-0 rounded-full px-1.5 py-0.5 font-mono text-caption-sm font-medium text-white/85 tabular-nums transition-colors hover:bg-white/20 hover:text-white focus-visible:ring-2 focus-visible:ring-white/50 focus-visible:outline-none"
      :aria-label="t('spaces.media.player.speed')"
      :title="t('spaces.media.player.speed')"
      @click="emit('cycleRate')"
    >
      {{ rate }}×
    </button>
    <button
      type="button"
      class="grid size-6 shrink-0 place-items-center rounded-full text-white/85 transition-colors hover:bg-white/20 hover:text-white focus-visible:ring-2 focus-visible:ring-white/50 focus-visible:outline-none"
      :aria-label="muted ? t('spaces.media.player.unmute') : t('spaces.media.player.mute')"
      :title="muted ? t('spaces.media.player.unmute') : t('spaces.media.player.mute')"
      @click="emit('toggleMute')"
    >
      <VolumeX v-if="muted" class="size-3.5" />
      <Volume2 v-else class="size-3.5" />
    </button>
  </div>
</template>
