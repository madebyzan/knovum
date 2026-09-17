<script setup lang="ts">
import { useI18n } from "vue-i18n";

import UiMarkdown from "../../../../../shared/ui/UiMarkdown.vue";
import type { SpaceMarkdownPayload } from "../../../model/nodes/node-payload";
import SpaceCanvasProcessingState from "./SpaceCanvasProcessingState.vue";

defineProps<{
  data: SpaceMarkdownPayload;
  detailLevel: "compact" | "summary" | "full";
}>();
const { t } = useI18n();
</script>

<template>
  <SpaceCanvasProcessingState
    v-if="data.derivationState === 'pending'"
    :label="t('spaces.creation.generatingDocument')"
  />
  <SpaceCanvasProcessingState
    v-else-if="data.derivationState === 'failed'"
    :label="t('spaces.creation.documentFailed')"
    failed
  />
  <UiMarkdown
    v-else-if="data.text"
    :content="data.text"
    class="min-h-0 flex-1 overflow-hidden px-3 py-2.5 text-caption leading-relaxed text-foreground-secondary"
    :class="detailLevel === 'full' ? 'line-clamp-8' : 'line-clamp-4'"
  />
  <p v-else class="min-h-0 flex-1 px-3 py-2.5 text-caption text-foreground-tertiary">
    {{ t("spaces.editor.emptyMarkdown") }}
  </p>
</template>
