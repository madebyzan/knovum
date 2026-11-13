<script setup lang="ts">
import { Captions, Scissors } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import ModelChoiceTrigger from "../../../../foundation/ai/ModelChoiceTrigger.vue";
import { useLocalModelOptions } from "../../../../foundation/ai/use-model-choice-options";
import type { LocalModelId } from "../../../../shared/contracts/generated/LocalModelId";
import SpaceCanvasActionTrigger from "./SpaceCanvasActionTrigger.vue";

defineProps<{ compact?: boolean; menuItem?: boolean; canAddSubtitles: boolean }>();
const emit = defineEmits<{ edit: []; addSubtitles: [model?: LocalModelId] }>();
const { t } = useI18n();
const { options: transcriptionOptions } = useLocalModelOptions(() => "transcription");
</script>

<template>
  <SpaceCanvasActionTrigger
    :label="t('spaces.media.videoEditor.edit')"
    :icon="Scissors"
    :compact="compact"
    :menu-item="menuItem"
    @invoke="emit('edit')"
  />
  <ModelChoiceTrigger
    :options="transcriptionOptions"
    :menu-item="menuItem"
    :disabled="!canAddSubtitles"
    @select="(id) => emit('addSubtitles', id as LocalModelId)"
  >
    <template #menu-trigger>
      <Captions class="size-3.5" />
      <span class="min-w-0 flex-1">{{ t("spaces.creation.video.subtitles.add") }}</span>
    </template>
    <template #trigger="{ multiple }">
      <SpaceCanvasActionTrigger
        :label="t('spaces.creation.video.subtitles.add')"
        :icon="Captions"
        :compact="compact"
        :menu-item="menuItem"
        :disabled="!canAddSubtitles"
        @invoke="!multiple && emit('addSubtitles')"
      />
    </template>
  </ModelChoiceTrigger>
</template>
