<script setup lang="ts">
import { Pause, Play, X } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import { formatDurationMs } from "../../media/format-duration";
import SpaceMediaTimeline from "./SpaceMediaTimeline.vue";

withDefaults(
  defineProps<{
    playing: boolean;
    currentMs: number;
    durationMs: number;
    density?: "compact" | "regular";
    floating?: boolean;
  }>(),
  { density: "regular" },
);
const emit = defineEmits<{ close: []; toggle: []; seek: [positionMs: number] }>();
const { t } = useI18n();
</script>

<template>
  <div
    class="nodrag pointer-events-auto flex flex-1 items-center"
    :class="[
      density === 'compact'
        ? 'min-h-(--knovum-control-height-sm) gap-1 px-1.5 py-0.5'
        : 'min-h-(--knovum-control-height-lg) gap-1.5 px-2 py-1',
      floating ? 'rounded-control border border-border/80 bg-chrome/95 shadow-popover' : 'bg-chrome/95',
    ]"
  >
    <UiIconButton
      :label="playing ? t('spaces.media.pause') : t('spaces.media.play')"
      size="sm"
      @click="emit('toggle')"
    >
      <Pause v-if="playing" class="size-(--knovum-icon-size-sm)" fill="currentColor" />
      <Play v-else class="ml-px size-(--knovum-icon-size-sm)" fill="currentColor" />
    </UiIconButton>
    <span
      class="text-right text-micro text-foreground-tertiary tabular-nums"
      :class="density === 'compact' ? 'w-8' : 'w-9'"
      >{{ formatDurationMs(currentMs) }}</span
    >
    <SpaceMediaTimeline :current-ms="currentMs" :duration-ms="durationMs" @seek="emit('seek', $event)" />
    <span
      class="text-micro text-foreground-tertiary tabular-nums"
      :class="density === 'compact' ? 'w-8' : 'w-9'"
      >{{ formatDurationMs(durationMs) }}</span
    >
    <UiIconButton :label="t('spaces.media.closePlayback')" size="sm" @click="emit('close')">
      <X class="size-(--knovum-icon-size-sm)" />
    </UiIconButton>
  </div>
</template>
