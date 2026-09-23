<script setup lang="ts">
import { Plus, Scissors, Trash2 } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import { formatDurationMs } from "../../media/format-duration";
import SpaceMediaControls from "../media/SpaceMediaControls.vue";

defineProps<{
  playing: boolean;
  currentMs: number;
  durationMs: number;
  canDelete: boolean;
}>();
defineEmits<{
  split: [];
  addText: [];
  remove: [];
  toggle: [];
  seek: [milliseconds: number];
}>();
const { t } = useI18n();
</script>

<template>
  <div class="flex h-11 shrink-0 items-center gap-1 border-t border-border bg-panel px-3">
    <UiButton size="sm" variant="ghost" @click="$emit('split')"
      ><Scissors class="size-(--knovum-icon-size-sm)" />{{ t("spaces.media.split") }}</UiButton
    >
    <UiButton size="sm" variant="ghost" @click="$emit('addText')"
      ><Plus class="size-(--knovum-icon-size-sm)" />{{ t("spaces.media.addText") }}</UiButton
    >
    <UiButton size="sm" variant="ghost" :disabled="!canDelete" @click="$emit('remove')"
      ><Trash2 class="size-(--knovum-icon-size-sm)" />{{ t("common.delete") }}</UiButton
    >
    <div class="mx-auto flex w-96 max-w-1/2 items-center gap-3">
      <SpaceMediaControls
        :playing="playing"
        :current-ms="currentMs"
        :duration-ms="durationMs"
        @toggle="$emit('toggle')"
        @seek="$emit('seek', $event)"
      />
    </div>
    <span class="text-caption text-foreground-secondary tabular-nums"
      >{{ formatDurationMs(currentMs) }} / {{ formatDurationMs(durationMs) }}</span
    >
  </div>
</template>
