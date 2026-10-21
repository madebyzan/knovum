<script setup lang="ts">
import { Info } from "@lucide/vue";
import { ToolbarButton } from "reka-ui";
import { useI18n } from "vue-i18n";

import UiIconButton from "../../../../../shared/ui/UiIconButton.vue";
import type { ExportTarget } from "../../../assets/export/target";
import SpaceCanvasExportMenu from "./SpaceCanvasExportMenu.vue";

defineProps<{ exportTarget: ExportTarget | undefined }>();
const emit = defineEmits<{ inspect: []; save: [format: string] }>();
const { t } = useI18n();
</script>

<template>
  <div class="flex h-full shrink-0 items-center gap-0.5 border-l border-stroke-divider px-1">
    <ToolbarButton as-child>
      <UiIconButton :label="t('spaces.inspector.title')" tooltip @click="emit('inspect')">
        <Info class="size-4.5" :stroke-width="1.6" />
      </UiIconButton>
    </ToolbarButton>
    <SpaceCanvasExportMenu :target="exportTarget" @save="emit('save', $event)" />
  </div>
</template>
