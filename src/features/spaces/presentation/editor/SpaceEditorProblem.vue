<script setup lang="ts">
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";

defineProps<{ error: "missing" | "conflict" }>();
defineEmits<{ discard: [] }>();
const { t } = useI18n();
const labels = { missing: "spaces.editor.tabs.missing", conflict: "spaces.editor.tabs.conflict" } as const;
</script>

<template>
  <div
    role="alert"
    class="flex shrink-0 items-center gap-3 border-b border-stroke-divider px-3 py-2 text-caption text-fg-danger"
  >
    <span class="flex-1">{{ t(labels[error]) }}</span>
    <UiButton size="sm" @click="$emit('discard')">{{ t("spaces.editor.tabs.discardClose") }}</UiButton>
  </div>
</template>
