<script setup lang="ts">
import { computed, toRef, useTemplateRef } from "vue";
import { useI18n } from "vue-i18n";

import type { PdfNormalizedRect } from "../../reading/pdf-rect-geometry";
import { usePdfAreaSelect } from "./use-pdf-area-select";

const props = defineProps<{
  areaMode: boolean;
  markupOpen: boolean;
}>();

const emit = defineEmits<{ areaCapture: [page: number, rect: PdfNormalizedRect] }>();
const { t } = useI18n();
const container = useTemplateRef<HTMLDivElement>("container");
const viewer = useTemplateRef<HTMLDivElement>("viewer");
const areaSelect = usePdfAreaSelect(container, toRef(props, "areaMode"), (page, rect) => {
  emit("areaCapture", page, rect);
});
const readerCssVars = computed(() => ({
  "--pdf-freetext-placeholder": JSON.stringify(t("spaces.inspector.reader.typeHere")),
}));

defineExpose({ container, viewer });
</script>

<template>
  <div
    class="relative min-h-0 min-w-0 flex-1"
    :data-reader-ignore-selection="markupOpen || areaMode ? '' : undefined"
  >
    <div
      ref="container"
      class="pdf-reader absolute inset-0 overflow-auto bg-surface-control/75"
      :class="areaMode && 'area-capture-active'"
      :style="readerCssVars"
      @mousedown="areaSelect.onMouseDown"
    >
      <div ref="viewer" class="pdfViewer" />
    </div>
    <div
      v-if="areaSelect.marquee.value"
      class="pointer-events-none absolute z-30 rounded-[2px] border-[1.5px] border-surface-accent bg-surface-accent/12"
      :style="{
        left: `${areaSelect.marquee.value.left}px`,
        top: `${areaSelect.marquee.value.top}px`,
        width: `${areaSelect.marquee.value.width}px`,
        height: `${areaSelect.marquee.value.height}px`,
      }"
    />
  </div>
</template>

<style scoped src="./pdf-editor.css"></style>
