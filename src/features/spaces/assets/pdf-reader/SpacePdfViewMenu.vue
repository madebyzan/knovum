<script setup lang="ts">
import { BookOpen, File, Maximize2, Minimize2, RotateCw, SlidersHorizontal } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiPopover from "../../../../shared/ui/UiPopover.vue";
import type { PdfSpreadMode } from "./pdf-editor-params";

defineProps<{
  disabled: boolean;
  spreadMode: PdfSpreadMode;
}>();

const emit = defineEmits<{
  setSpread: [mode: PdfSpreadMode];
  fitWidth: [];
  fitPage: [];
  rotate: [];
}>();

const { t } = useI18n();
</script>

<template>
  <UiPopover align="end" content-class="w-44 p-1">
    <template #trigger>
      <UiIconButton :label="t('spaces.inspector.reader.viewOptions')" size="sm" tooltip :disabled="disabled">
        <SlidersHorizontal class="size-3.5" />
      </UiIconButton>
    </template>
    <div class="flex flex-col gap-0.5 text-caption">
      <button
        type="button"
        class="flex items-center gap-2 rounded-md px-2 py-1.5 text-left transition-colors hover:bg-surface-control-hover"
        :class="spreadMode === 0 ? 'bg-surface-control font-medium text-fg-primary' : 'text-fg-secondary'"
        @click="emit('setSpread', 0)"
      >
        <File class="size-3.5" />
        <span>{{ t("spaces.inspector.reader.singlePage") }}</span>
      </button>
      <button
        type="button"
        class="flex items-center gap-2 rounded-md px-2 py-1.5 text-left transition-colors hover:bg-surface-control-hover"
        :class="spreadMode === 1 ? 'bg-surface-control font-medium text-fg-primary' : 'text-fg-secondary'"
        @click="emit('setSpread', 1)"
      >
        <BookOpen class="size-3.5" />
        <span>{{ t("spaces.inspector.reader.twoPageSpread") }}</span>
      </button>
      <div class="my-0.5 h-px bg-stroke-divider" />
      <button
        type="button"
        class="flex items-center gap-2 rounded-md px-2 py-1.5 text-left text-fg-secondary transition-colors hover:bg-surface-control-hover hover:text-fg-primary"
        @click="emit('fitWidth')"
      >
        <Maximize2 class="size-3.5" />
        <span>{{ t("spaces.inspector.pdfReader.fitWidth") }}</span>
      </button>
      <button
        type="button"
        class="flex items-center gap-2 rounded-md px-2 py-1.5 text-left text-fg-secondary transition-colors hover:bg-surface-control-hover hover:text-fg-primary"
        @click="emit('fitPage')"
      >
        <Minimize2 class="size-3.5" />
        <span>{{ t("spaces.inspector.reader.fitPage") }}</span>
      </button>
      <button
        type="button"
        class="flex items-center gap-2 rounded-md px-2 py-1.5 text-left text-fg-secondary transition-colors hover:bg-surface-control-hover hover:text-fg-primary"
        @click="emit('rotate')"
      >
        <RotateCw class="size-3.5" />
        <span>{{ t("spaces.inspector.reader.rotateClockwise") }}</span>
      </button>
    </div>
  </UiPopover>
</template>
