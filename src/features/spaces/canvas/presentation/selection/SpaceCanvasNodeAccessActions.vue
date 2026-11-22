<script setup lang="ts">
import { BookOpen, Info } from "@lucide/vue";
import { ToolbarButton } from "reka-ui";
import { useI18n } from "vue-i18n";

import UiIconButton from "../../../../../shared/ui/UiIconButton.vue";
import type { ExportTarget } from "../../../assets/export/target";
import type { NodeReaderAction } from "../../../reading/use-node-reader-action";
import SpaceCanvasExportMenu from "./SpaceCanvasExportMenu.vue";

/** How a node is read, inspected, or exported directly from its canvas action bar. */
withDefaults(
  defineProps<{
    exportTarget: ExportTarget | undefined;
    readerAction?: NodeReaderAction | undefined;
    showInfo?: boolean;
  }>(),
  { showInfo: true },
);
const emit = defineEmits<{ inspect: []; save: [format: string] }>();
const { t } = useI18n();
</script>

<template>
  <div class="flex h-full shrink-0 items-center gap-0.5 border-l border-stroke-divider px-1">
    <slot />
    <ToolbarButton v-if="readerAction" as-child>
      <UiIconButton :label="t(readerAction.labelKey)" tooltip @click="readerAction.run()">
        <BookOpen class="size-4" :stroke-width="1.6" />
      </UiIconButton>
    </ToolbarButton>
    <ToolbarButton v-else-if="showInfo" as-child>
      <UiIconButton :label="t('spaces.inspector.title')" tooltip @click="emit('inspect')">
        <Info class="size-4" :stroke-width="1.6" />
      </UiIconButton>
    </ToolbarButton>
    <SpaceCanvasExportMenu :target="exportTarget" @save="emit('save', $event)" />
  </div>
</template>
