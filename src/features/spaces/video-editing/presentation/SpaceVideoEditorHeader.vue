<script setup lang="ts">
import { Save, Sparkles } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";

defineProps<{
  title: string;
  canSave: boolean;
  canRender: boolean;
  dirty: boolean;
  preparing: boolean;
}>();
defineEmits<{ save: []; render: [] }>();
const { t } = useI18n();
</script>

<template>
  <header class="flex h-11 shrink-0 items-center gap-2 border-b border-stroke-default bg-surface-panel px-3">
    <div class="min-w-0 flex-1">
      <h1 class="truncate text-label font-semibold">
        {{ title }}<span v-if="dirty" class="ml-1 text-fg-tertiary" aria-hidden="true">•</span>
        <span v-if="dirty" class="sr-only">{{ t("spaces.media.videoEditor.unsaved.status") }}</span>
      </h1>
      <p class="text-caption-sm text-fg-tertiary">
        {{ t(preparing ? "spaces.media.videoEditor.preview.optimizing" : "spaces.media.videoEditor.title") }}
      </p>
    </div>
    <UiButton size="sm" :disabled="!canSave" @click="$emit('save')"
      ><Save class="size-3.5" />{{ t("common.save") }}</UiButton
    >
    <UiButton size="sm" variant="primary" :disabled="!canRender" @click="$emit('render')"
      ><Sparkles class="size-3.5" />{{ t("spaces.media.videoEditor.render.action") }}</UiButton
    >
  </header>
</template>
