<script setup lang="ts">
import { Save, Sparkles } from "@lucide/vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiSpinner from "../../../../shared/ui/UiSpinner.vue";
import { formatVideoEditTime } from "../timecode";
import { projectDurationMs } from "../timing";
import SpaceVideoOutputSettings from "./SpaceVideoOutputSettings.vue";
import type { VideoEditor } from "./use-video-editor";

const props = defineProps<{ editor: VideoEditor }>();
const { t } = useI18n();
const { data, project, history, actions, renderable, commit } = props.editor;
const canSave = computed(
  () =>
    !props.editor.session.error.value &&
    !!project.value &&
    history.dirty.value &&
    !actions.saving.value &&
    !actions.rendering.value,
);
const durationMs = computed(() => (project.value ? projectDurationMs(project.value) : 0));
</script>

<template>
  <header
    :aria-label="t('spaces.media.videoEditor.title')"
    class="flex h-11 shrink-0 items-center gap-2 border-b border-stroke-default bg-surface-panel px-3"
  >
    <div class="flex min-w-0 flex-1 items-baseline gap-2">
      <h1 class="truncate text-label font-semibold">{{ data.title }}</h1>
      <span v-if="history.dirty.value" class="text-caption text-fg-tertiary">{{
        t("spaces.media.videoEditor.unsaved.status")
      }}</span>
      <span class="text-caption text-fg-tertiary tabular-nums">{{ formatVideoEditTime(durationMs) }}</span>
    </div>
    <SpaceVideoOutputSettings v-if="project" :project="project" @edit="commit" />
    <UiButton size="sm" :disabled="!canSave" @click="actions.save()">
      <UiSpinner v-if="actions.saving.value" class="size-3.5" />
      <Save v-else class="size-3.5" />{{ t("common.save") }}
    </UiButton>
    <UiButton
      size="sm"
      variant="primary"
      :disabled="!renderable || actions.rendering.value"
      @click="actions.render"
    >
      <Sparkles class="size-3.5" />{{ t("spaces.media.videoEditor.render.action") }}
    </UiButton>
  </header>
</template>
