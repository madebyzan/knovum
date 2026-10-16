<script setup lang="ts">
import { Pause, Play } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import { formatDurationMs } from "../format-duration";
import SpaceMediaTimeline from "./SpaceMediaTimeline.vue";

withDefaults(
  defineProps<{
    playing: boolean;
    currentMs: number;
    durationMs: number;
    density?: "compact" | "regular";
  }>(),
  { density: "regular" },
);
const emit = defineEmits<{ toggle: []; seek: [positionMs: number] }>();
const { t } = useI18n();
</script>

<template>
  <div
    class="nodrag pointer-events-auto flex items-center rounded-full border border-white/15 bg-black/75 text-white shadow-dialog backdrop-blur-md select-none"
    :class="density === 'compact' ? 'h-7 gap-2 px-2.5' : 'h-8 gap-2.5 px-3'"
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
  </div>
</template>
