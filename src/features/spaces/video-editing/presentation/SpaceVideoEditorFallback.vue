<script setup lang="ts">
import { AlertTriangle } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiSpinner from "../../../../shared/ui/UiSpinner.vue";
import type { VideoEditor } from "./use-video-editor";

const props = defineProps<{ editor: VideoEditor }>();
const { t } = useI18n();
const { bootstrap, session } = props.editor;
</script>

<template>
  <div class="grid min-h-0 flex-1 place-content-center justify-items-center gap-3 px-8 text-center">
    <template v-if="session.obsolete.value">
      <AlertTriangle class="size-7 text-fg-warning" />
      <h2 class="text-title-sm font-semibold">
        {{ t("spaces.media.videoEditor.obsolete.title") }}
      </h2>
      <p class="max-w-md text-body text-fg-secondary">
        {{ t("spaces.media.videoEditor.obsolete.description") }}
      </p>
      <UiButton variant="primary" size="sm" @click="bootstrap.reset">
        {{ t("spaces.media.videoEditor.obsolete.action") }}
      </UiButton>
    </template>
    <template v-else-if="bootstrap.error.value">
      <AlertTriangle class="size-7 text-fg-danger" />
      <p class="max-w-md text-body text-fg-danger">{{ bootstrap.error.value }}</p>
      <UiButton size="sm" @click="bootstrap.retry">
        {{ t("spaces.media.videoEditor.preview.retry") }}
      </UiButton>
    </template>
    <template v-else>
      <UiSpinner class="size-5 text-fg-secondary" />
      <p class="text-body text-fg-secondary">{{ t("spaces.media.videoEditor.loading") }}</p>
    </template>
  </div>
</template>
