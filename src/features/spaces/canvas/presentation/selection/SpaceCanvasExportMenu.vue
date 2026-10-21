<script setup lang="ts">
import { Download } from "@lucide/vue";
import { ToolbarButton } from "reka-ui";
import { useI18n } from "vue-i18n";

import UiIconButton from "../../../../../shared/ui/UiIconButton.vue";
import type { ExportTarget } from "../../../assets/export/target";
import SpaceExportMenu from "../../../assets/presentation/export/SpaceExportMenu.vue";

defineProps<{ target: ExportTarget | undefined }>();
const emit = defineEmits<{ save: [format: string] }>();
const { t } = useI18n();
</script>

<template>
  <SpaceExportMenu :target="target" side="top" align="center" @save="emit('save', $event)">
    <template #trigger>
      <ToolbarButton as-child>
        <UiIconButton :label="t('spaces.canvas.nodeActions.export')" tooltip :disabled="!target">
          <Download class="size-4.5" :stroke-width="1.6" />
        </UiIconButton>
      </ToolbarButton>
    </template>
  </SpaceExportMenu>
</template>
