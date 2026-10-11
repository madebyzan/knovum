<script setup lang="ts">
import { Download, Info } from "@lucide/vue";
import { ToolbarButton } from "reka-ui";
import { useI18n } from "vue-i18n";

import UiIconButton from "../../../../../shared/ui/UiIconButton.vue";

defineProps<{ disabled?: boolean }>();
const emit = defineEmits<{ inspect: []; export: [] }>();
const { t } = useI18n();
</script>

<template>
  <div class="flex h-full shrink-0 items-center gap-0.5 border-l border-stroke-divider px-1">
    <ToolbarButton as-child>
      <UiIconButton :label="t('spaces.inspector.title')" tooltip @click="emit('inspect')">
        <Info class="size-4.5" :stroke-width="1.6" />
      </UiIconButton>
    </ToolbarButton>
    <ToolbarButton as-child>
      <UiIconButton
        :label="t('spaces.canvas.nodeActions.export')"
        :disabled="disabled"
        tooltip
        @click="emit('export')"
      >
        <Download class="size-4.5" :stroke-width="1.6" />
      </UiIconButton>
    </ToolbarButton>
  </div>
</template>
