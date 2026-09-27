<script setup lang="ts">
import { Focus, X } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import UiIconButton from "../../../../shared/ui/UiIconButton.vue";

defineProps<{ count: number }>();
const emit = defineEmits<{ exit: [] }>();
const { t } = useI18n();
</script>

<template>
  <div
    class="flex h-8 items-center gap-2 rounded-md border border-stroke-default bg-surface-overlay pr-0.5 pl-2.5 text-caption text-fg-secondary select-none"
    role="status"
  >
    <Focus class="size-3" :stroke-width="1.6" />
    <span>{{ t("spaces.scenes.focus.mode", { count }) }}</span>
    <UiIconButton :label="t('spaces.scenes.focus.exit')" size="sm" @click="emit('exit')">
      <X class="size-3" :stroke-width="1.7" />
    </UiIconButton>
  </div>
</template>
