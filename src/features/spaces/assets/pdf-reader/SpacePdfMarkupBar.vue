<script setup lang="ts">
import {
  Check,
  Highlighter,
  MousePointer2,
  PenTool,
  Redo2,
  RotateCcw,
  Save,
  Trash2,
  Type,
  Undo2,
} from "@lucide/vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import {
  PDF_HIGHLIGHT_SWATCHES,
  PDF_INK_SWATCHES,
  PDF_INK_WIDTHS,
  PDF_TEXT_SIZES,
  type PdfReaderSession,
} from "./pdf-editor-params";
import type { PdfMarkupState } from "./use-pdf-markup";

const MARKUP_TOOLS = [
  { id: "none" as const, icon: MousePointer2, labelKey: undefined },
  { id: "highlight" as const, icon: Highlighter, labelKey: "spaces.inspector.reader.highlight" },
  { id: "freetext" as const, icon: Type, labelKey: "spaces.inspector.reader.addText" },
  { id: "ink" as const, icon: PenTool, labelKey: "spaces.inspector.reader.draw" },
] as const;

defineProps<{
  markup: PdfMarkupState;
  session?: PdfReaderSession | undefined;
  modified: boolean;
  saving: boolean;
}>();

const emit = defineEmits<{ discard: []; save: []; done: [] }>();

const { t } = useI18n();
</script>

<template>
  <div
    data-reader-ignore-selection
    class="flex min-h-9 shrink-0 flex-wrap items-center justify-between gap-x-2 gap-y-1 border-b border-stroke-divider bg-surface-raised px-2 py-1 select-none"
  >
    <div class="flex flex-wrap items-center gap-1.5">
      <div class="inline-flex rounded-md border border-stroke-divider bg-surface-control p-0.5">
        <UiIconButton
          v-for="item in MARKUP_TOOLS"
          :key="item.id"
          :label="item.labelKey ? t(item.labelKey) : 'Select'"
          size="sm"
          :tooltip="Boolean(item.labelKey)"
          :class="
            markup.tool.value === item.id &&
            (item.id === 'none'
              ? 'bg-surface-raised text-fg-primary shadow-2xs'
              : 'bg-surface-raised text-fg-accent shadow-2xs')
          "
          @click="markup.selectTool(item.id)"
        >
          <component :is="item.icon" class="size-3.5" />
        </UiIconButton>
      </div>

      <div v-if="markup.tool.value === 'highlight'" class="flex items-center gap-1">
        <button
          v-for="hex in PDF_HIGHLIGHT_SWATCHES"
          :key="hex"
          type="button"
          class="size-4 rounded-full border transition-transform hover:scale-110"
          :class="
            markup.highlightColor.value === hex
              ? 'border-fg-primary ring-2 ring-stroke-focus/50'
              : 'border-black/15'
          "
          :style="{ backgroundColor: hex }"
          @click="markup.setHighlightColor(hex)"
        />
      </div>

      <template v-else-if="markup.tool.value === 'freetext' || markup.tool.value === 'ink'">
        <div class="flex items-center gap-1">
          <button
            v-for="hex in PDF_INK_SWATCHES"
            :key="hex"
            type="button"
            class="size-3.5 rounded-full border transition-transform hover:scale-110"
            :class="
              markup.inkColor.value === hex
                ? 'border-fg-primary ring-2 ring-stroke-focus/50'
                : 'border-black/15'
            "
            :style="{ backgroundColor: hex }"
            @click="markup.setInkColor(hex)"
          />
        </div>
        <span class="h-3.5 w-px bg-stroke-divider" />
        <div v-if="markup.tool.value === 'freetext'" class="inline-flex items-center gap-0.5">
          <button
            v-for="size in PDF_TEXT_SIZES"
            :key="size"
            type="button"
            class="rounded-sm px-1 py-0.5 text-[11px] tabular-nums transition-colors"
            :class="
              markup.textSize.value === size
                ? 'bg-surface-control font-semibold text-fg-primary'
                : 'text-fg-secondary hover:bg-surface-control-hover'
            "
            @click="markup.setTextSize(size)"
          >
            {{ size }}
          </button>
        </div>
        <div v-else class="inline-flex items-center gap-0.5">
          <button
            v-for="width in PDF_INK_WIDTHS"
            :key="width"
            type="button"
            class="grid size-5 place-items-center rounded-sm transition-colors"
            :class="
              markup.inkWidth.value === width
                ? 'bg-surface-control text-fg-primary'
                : 'text-fg-secondary hover:bg-surface-control-hover'
            "
            @click="markup.setInkWidth(width)"
          >
            <span
              class="rounded-full bg-current"
              :style="{ width: `${width + 3}px`, height: `${width + 3}px` }"
            />
          </button>
        </div>
      </template>
    </div>

    <div class="ml-auto flex shrink-0 items-center gap-1">
      <UiIconButton :label="t('common.delete')" size="sm" tooltip @click="session?.deleteSelected()">
        <Trash2 class="size-3.5" />
      </UiIconButton>
      <UiIconButton :label="t('common.undo')" size="sm" tooltip @click="session?.undo()">
        <Undo2 class="size-3.5" />
      </UiIconButton>
      <UiIconButton :label="t('common.redo')" size="sm" tooltip @click="session?.redo()">
        <Redo2 class="size-3.5" />
      </UiIconButton>
      <template v-if="modified">
        <span class="mx-0.5 h-3.5 w-px bg-stroke-divider" />
        <UiIconButton
          :label="t('common.discardChanges')"
          size="sm"
          tooltip
          :disabled="saving"
          @click="emit('discard')"
        >
          <RotateCcw class="size-3.5" />
        </UiIconButton>
        <UiButton variant="primary" size="sm" class="gap-1" :disabled="saving" @click="emit('save')">
          <Save class="size-3" />
          <span>{{ saving ? t("common.saving") : t("common.save") }}</span>
        </UiButton>
      </template>
      <UiButton v-else variant="secondary" size="sm" class="gap-1" @click="emit('done')">
        <Check class="size-3" />
        <span>{{ t("common.done") }}</span>
      </UiButton>
    </div>
  </div>
</template>
