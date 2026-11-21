<script setup lang="ts">
import { Crop, PanelLeft, PenTool, Search, ZoomIn, ZoomOut } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import type { PdfReaderSession, PdfSpreadMode } from "./pdf-editor-params";
import SpacePdfPageControl from "./SpacePdfPageControl.vue";
import SpacePdfViewMenu from "./SpacePdfViewMenu.vue";

defineProps<{
  session?: PdfReaderSession | undefined;
  sidebarOpen: boolean;
  findOpen: boolean;
  areaMode: boolean;
  markupOpen: boolean;
  canMarkup: boolean;
  modified: boolean;
  currentPage: number;
  pageCount: number;
  scalePercent: number;
  spreadMode: PdfSpreadMode;
}>();

const emit = defineEmits<{
  toggleSidebar: [];
  toggleFind: [];
  toggleArea: [];
  toggleMarkup: [];
  setSpread: [mode: PdfSpreadMode];
}>();

const { t } = useI18n();
</script>

<template>
  <div
    data-reader-ignore-selection
    class="flex h-8.5 shrink-0 items-center justify-between gap-2 border-b border-stroke-divider/70 bg-surface-canvas px-3 select-none"
  >
    <div class="flex min-w-0 items-center gap-1">
      <UiIconButton
        :label="t('spaces.inspector.reader.thumbnails')"
        size="sm"
        tooltip
        :disabled="!session"
        :class="sidebarOpen && 'bg-surface-control text-fg-primary'"
        @click="emit('toggleSidebar')"
      >
        <PanelLeft class="size-3.5" :stroke-width="1.6" />
      </UiIconButton>
      <SpacePdfPageControl
        v-if="pageCount"
        :current-page="currentPage"
        :page-count="pageCount"
        :ready="Boolean(session)"
        @go-to-page="session?.goToPage($event)"
      />
    </div>

    <div class="flex shrink-0 items-center gap-0.5">
      <UiIconButton
        :label="t('spaces.inspector.reader.searchPlaceholder')"
        size="sm"
        tooltip
        :disabled="!session"
        :class="findOpen && 'bg-surface-control text-fg-primary'"
        @click="emit('toggleFind')"
      >
        <Search class="size-3.5" :stroke-width="1.6" />
      </UiIconButton>
      <UiIconButton
        v-if="canMarkup"
        :label="t('spaces.inspector.reader.areaCapture')"
        size="sm"
        tooltip
        :disabled="!session"
        :class="areaMode && 'bg-surface-accent-subtle text-fg-accent'"
        @click="emit('toggleArea')"
      >
        <Crop class="size-3.5" :stroke-width="1.6" />
      </UiIconButton>
      <UiIconButton
        v-if="canMarkup"
        :label="t('spaces.inspector.reader.markup')"
        size="sm"
        tooltip
        :disabled="!session"
        class="relative"
        :class="markupOpen && 'bg-surface-accent-subtle text-fg-accent'"
        @click="emit('toggleMarkup')"
      >
        <PenTool class="size-3.5" :stroke-width="1.6" />
        <span v-if="modified" class="absolute top-1 right-1 size-1.5 rounded-full bg-surface-accent" />
      </UiIconButton>
      <span class="mx-0.5 h-3.5 w-px bg-stroke-divider" />
      <UiIconButton
        :label="t('spaces.inspector.pdfReader.zoomOut')"
        size="sm"
        tooltip
        :disabled="!session"
        @click="session?.zoom(-1)"
      >
        <ZoomOut class="size-3.5" :stroke-width="1.6" />
      </UiIconButton>
      <button
        type="button"
        class="min-w-9 rounded-sm px-1 py-0.5 text-center text-caption-sm text-fg-secondary tabular-nums hover:bg-surface-control-hover hover:text-fg-primary"
        :disabled="!session"
        @click="session?.fitWidth()"
      >
        {{ scalePercent }}%
      </button>
      <UiIconButton
        :label="t('spaces.inspector.pdfReader.zoomIn')"
        size="sm"
        tooltip
        :disabled="!session"
        @click="session?.zoom(1)"
      >
        <ZoomIn class="size-3.5" :stroke-width="1.6" />
      </UiIconButton>
      <SpacePdfViewMenu
        :disabled="!session"
        :spread-mode="spreadMode"
        @set-spread="emit('setSpread', $event)"
        @fit-width="session?.fitWidth()"
        @fit-page="session?.fitPage()"
        @rotate="session?.rotate(90)"
      />
    </div>
  </div>
</template>
