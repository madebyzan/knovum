<script setup lang="ts">
import { Film, Music2, Type } from "@lucide/vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import type { VideoEditProject } from "../project";
import { rowOffsets, TIMELINE_ROWS } from "../timeline/track-geometry";

const props = defineProps<{ project: VideoEditProject }>();
const { t } = useI18n();
const rows = computed(() => rowOffsets(props.project));
</script>

<template>
  <div class="w-24 shrink-0 border-r border-stroke-divider bg-surface-panel text-caption text-fg-secondary">
    <div :style="{ height: `${TIMELINE_ROWS.ruler}px` }" class="border-b border-stroke-divider" />
    <div
      class="flex items-center gap-1.5 border-b border-stroke-divider px-2"
      :style="{ height: `${TIMELINE_ROWS.text}px` }"
    >
      <Type class="size-3.5 shrink-0" />{{ t("spaces.media.videoEditor.tracks.text") }}
    </div>
    <div
      class="flex items-center gap-1.5 border-b border-stroke-divider px-2"
      :style="{ height: `${TIMELINE_ROWS.video}px` }"
    >
      <Film class="size-3.5 shrink-0" />{{ t("spaces.media.videoEditor.tracks.video") }}
    </div>
    <div
      v-for="lane in rows.lanes"
      :key="lane"
      class="flex items-center gap-1.5 border-b border-stroke-divider px-2"
      :style="{ height: `${TIMELINE_ROWS.audio}px` }"
    >
      <Music2 class="size-3.5 shrink-0" />{{ t("spaces.media.videoEditor.tracks.audio") }} {{ lane }}
    </div>
  </div>
</template>
