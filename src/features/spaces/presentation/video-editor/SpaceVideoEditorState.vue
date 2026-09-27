<script setup lang="ts">
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import type { SpaceVideoPayload } from "../../model/nodes/node-payload";
import { versionedMediaSource } from "../media/media-element-source";

defineProps<{
  data: SpaceVideoPayload;
  proxyPath?: string | undefined;
  error?: string | undefined;
  canRetry: boolean;
}>();
defineEmits<{ metadataReady: [event: Event]; metadataError: []; retry: [] }>();
const { t } = useI18n();
</script>

<template>
  <div
    v-if="error"
    class="grid min-h-0 flex-1 place-content-center justify-items-center gap-3 px-6 text-body-sm text-fg-danger"
  >
    <p class="max-w-xl text-center">{{ error }}</p>
    <UiButton v-if="canRetry" size="sm" @click="$emit('retry')">{{
      t("spaces.media.videoEditor.preview.retry")
    }}</UiButton>
  </div>
  <div v-else class="grid min-h-0 flex-1 place-items-center text-body-sm text-fg-secondary">
    <video
      class="pointer-events-none absolute size-px opacity-0"
      :src="versionedMediaSource(proxyPath ?? data.assetPath, data.sourceRevision)"
      preload="metadata"
      aria-hidden="true"
      tabindex="-1"
      @loadedmetadata="$emit('metadataReady', $event)"
      @error="$emit('metadataError')"
    />
    {{ t("spaces.media.videoEditor.loading") }}
  </div>
</template>
