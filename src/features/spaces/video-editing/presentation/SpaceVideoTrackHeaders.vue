<script setup lang="ts">
import { Film, Music2, Plus, Type } from "@lucide/vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import type { VideoEditProject } from "../project";
import { rowOffsets, TIMELINE_ROWS } from "../timeline/track-geometry";
import type { VideoEditor } from "./use-video-editor";

const props = defineProps<{ project: VideoEditProject; editor?: VideoEditor | undefined }>();
const { t } = useI18n();
const rows = computed(() => rowOffsets(props.project));
</script>

<template>
  <div
    class="w-26 shrink-0 border-r border-stroke-divider bg-surface-panel text-caption text-fg-secondary select-none"
  >
    <div
      :style="{ height: `${TIMELINE_ROWS.ruler}px` }"
      class="sticky top-0 z-10 border-b border-stroke-divider bg-surface-panel"
    />
    <div
      class="flex items-center gap-1.5 border-b border-stroke-divider px-2"
      :style="{ height: `${TIMELINE_ROWS.text}px` }"
    >
      <Type class="size-3.5 shrink-0 text-fg-tertiary" />
      <span class="min-w-0 flex-1 truncate">{{ t("spaces.media.videoEditor.tracks.text") }}</span>
      <UiIconButton
        v-if="editor"
        size="sm"
        class="size-5"
        tooltip
        :label="t('spaces.media.videoEditor.text.add')"
        @click="editor.actions.addText()"
      >
        <Plus class="size-3" />
      </UiIconButton>
    </div>
    <div
      class="flex items-center gap-1.5 border-b border-stroke-divider px-2"
      :style="{ height: `${TIMELINE_ROWS.video}px` }"
    >
      <Film class="size-3.5 shrink-0 text-fg-tertiary" />
      <span class="truncate">{{ t("spaces.media.videoEditor.tracks.video") }}</span>
    </div>
    <div
      v-for="lane in rows.lanes"
      :key="lane"
      class="flex items-center gap-1.5 border-b border-stroke-divider px-2"
      :style="{ height: `${TIMELINE_ROWS.audio}px` }"
    >
      <Music2 class="size-3.5 shrink-0 text-fg-tertiary" />
      <span class="truncate">
        {{ t("spaces.media.videoEditor.tracks.audio")
        }}<span v-if="rows.lanes > 1" class="tabular-nums">&nbsp;{{ lane }}</span>
      </span>
    </div>
  </div>
</template>
