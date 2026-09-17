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
    class="flex h-8 items-center gap-2 rounded-control border border-border bg-overlay pr-0.5 pl-2.5 text-caption text-foreground-secondary select-none"
    role="status"
  >
    <Focus class="size-(--knovum-icon-size-xs)" :stroke-width="1.6" />
    <span>{{ t("spaces.navigation.focusedView", { count }) }}</span>
    <UiIconButton :label="t('spaces.navigation.exitFocusedView')" size="sm" @click="emit('exit')">
      <X class="size-(--knovum-icon-size-xs)" :stroke-width="1.7" />
    </UiIconButton>
  </div>
</template>
