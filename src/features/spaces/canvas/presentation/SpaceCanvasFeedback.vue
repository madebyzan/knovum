<script setup lang="ts">
import { AlertCircle, X } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import SpaceCanvasSceneFeedback from "../../scenes/presentation/SpaceCanvasSceneFeedback.vue";
import SpaceEmptyState from "../../workspace/presentation/SpaceEmptyState.vue";

defineProps<{
  empty: boolean;
  error?: string;
  fileHover: boolean;
  fileHoverPosition?: { x: number; y: number };
}>();
const emit = defineEmits<{ dismissError: [] }>();
const { t } = useI18n();
</script>

<template>
  <SpaceCanvasSceneFeedback />
  <SpaceEmptyState v-if="empty" />
  <div
    v-if="fileHover && fileHoverPosition"
    class="pointer-events-none fixed z-40 -translate-y-full pb-3 pl-3"
    :style="{ left: `${fileHoverPosition.x}px`, top: `${fileHoverPosition.y}px` }"
  >
    <div
      class="rounded-md border border-stroke-default bg-surface-overlay px-3 py-1.5 text-label font-medium"
    >
      {{ t("spaces.canvas.dropToSave") }}
    </div>
  </div>
  <div
    v-if="error"
    class="pointer-events-auto absolute top-4 left-1/2 flex max-w-sm -translate-x-1/2 items-center gap-2 rounded-md border border-stroke-default bg-surface-overlay py-1 pr-0.5 pl-2.5 text-caption text-fg-secondary"
    role="alert"
    aria-atomic="true"
  >
    <AlertCircle class="size-3 shrink-0" :stroke-width="1.6" />
    <span class="line-clamp-2 min-w-0 flex-1 wrap-break-word">{{ error }}</span>
    <UiIconButton :label="t('common.close')" size="sm" @click="emit('dismissError')">
      <X class="size-3" />
    </UiIconButton>
  </div>
</template>
