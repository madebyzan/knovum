<script setup lang="ts">
import { FileText, Maximize2, RotateCw, ZoomIn, ZoomOut } from "@lucide/vue";
import { convertFileSrc } from "@tauri-apps/api/core";
import { computed, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import type { KnowledgeChunk } from "../../knowledge/model/schema/persisted-chunk";
import SpaceTextQuote from "../../knowledge/presentation/SpaceTextQuote.vue";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceImagePayload } from "../../model/nodes/node-payload";

const MIN_ZOOM = 0.25;
const MAX_ZOOM = 4;

const props = defineProps<{
  node: SpaceCanvasNode<SpaceImagePayload>;
  chunks: readonly KnowledgeChunk[];
  recognizedText: (resourceId: string) => string | undefined;
  quote?: string | undefined;
}>();

const { t } = useI18n();
const zoom = shallowRef(1);
const fitMode = shallowRef(true);
const rotation = shallowRef(0);
const showOcr = shallowRef(true);
const loadedWidth = shallowRef<number>();

const imageSrc = computed(() => convertFileSrc(props.node.data.assetPath));
const baseWidth = computed(() => props.node.data.intrinsicWidth ?? loadedWidth.value);
const dimensionsLabel = computed(() => {
  const { intrinsicWidth: w, intrinsicHeight: h } = props.node.data;
  return w && h ? `${Math.round(w)} × ${Math.round(h)}` : undefined;
});
const ocrPassages = computed(() => {
  const direct = props.recognizedText(props.node.id)?.trim();
  const fromChunks = props.chunks
    .filter((chunk) => chunk.kind !== "node")
    .map((chunk) => chunk.text.trim())
    .filter(Boolean);
  if (fromChunks.length) return fromChunks;
  return direct ? [direct] : [];
});

function stepZoom(delta: number) {
  fitMode.value = false;
  zoom.value = Math.min(MAX_ZOOM, Math.max(MIN_ZOOM, Number((zoom.value + delta).toFixed(2))));
}

function resetFit() {
  fitMode.value = true;
  zoom.value = 1;
}

function onKeyDown(event: KeyboardEvent) {
  if (!(event.metaKey || event.ctrlKey)) return;
  if (event.key === "=" || event.key === "+") {
    event.preventDefault();
    stepZoom(0.25);
  } else if (event.key === "-") {
    event.preventDefault();
    stepZoom(-0.25);
  } else if (event.key === "0") {
    event.preventDefault();
    resetFit();
  }
}

function onImageLoad(event: Event) {
  const img = event.target as HTMLImageElement | null;
  if (img?.naturalWidth) loadedWidth.value = img.naturalWidth;
}

watch(
  () => props.node.id,
  () => {
    fitMode.value = true;
    zoom.value = 1;
    rotation.value = 0;
    loadedWidth.value = undefined;
  },
);
</script>

<template>
  <div class="flex h-full min-h-0 flex-col bg-surface-canvas outline-none" tabindex="-1" @keydown="onKeyDown">
    <div
      data-reader-ignore-selection
      class="flex h-9 shrink-0 items-center justify-between gap-2 border-b border-stroke-divider bg-surface-panel px-2.5 select-none"
    >
      <span class="truncate text-caption-sm text-fg-tertiary tabular-nums">
        {{ dimensionsLabel ?? node.data.fileName }}
      </span>
      <div class="flex shrink-0 items-center gap-0.5">
        <UiIconButton
          v-if="ocrPassages.length"
          :label="t('spaces.inspector.extractedDocumentText')"
          size="sm"
          tooltip
          :class="showOcr && 'bg-surface-control text-fg-primary'"
          @click="showOcr = !showOcr"
        >
          <FileText class="size-3.5" />
        </UiIconButton>
        <UiIconButton
          :label="t('spaces.inspector.pdfReader.zoomOut')"
          size="sm"
          tooltip
          @click="stepZoom(-0.25)"
        >
          <ZoomOut class="size-3.5" />
        </UiIconButton>
        <button
          type="button"
          class="min-w-10 rounded-sm px-1 py-0.5 text-center text-caption-sm text-fg-secondary tabular-nums hover:bg-surface-control-hover hover:text-fg-primary"
          @click="resetFit"
        >
          {{ fitMode ? "Fit" : `${Math.round(zoom * 100)}%` }}
        </button>
        <UiIconButton
          :label="t('spaces.inspector.pdfReader.zoomIn')"
          size="sm"
          tooltip
          @click="stepZoom(0.25)"
        >
          <ZoomIn class="size-3.5" />
        </UiIconButton>
        <UiIconButton :label="t('spaces.inspector.pdfReader.fitWidth')" size="sm" tooltip @click="resetFit">
          <Maximize2 class="size-3.5" />
        </UiIconButton>
        <UiIconButton
          :label="t('spaces.inspector.reader.rotateClockwise')"
          size="sm"
          tooltip
          @click="rotation = (rotation + 90) % 360"
        >
          <RotateCw class="size-3.5" />
        </UiIconButton>
      </div>
    </div>

    <div
      data-reader-ignore-selection
      class="flex min-h-0 flex-1 overflow-auto bg-surface-control/75 p-4 select-none"
    >
      <img
        :src="imageSrc"
        :alt="node.data.title"
        class="m-auto rounded-[3px] border border-stroke-divider/70 bg-white object-contain shadow-sm transition-transform duration-150"
        :class="fitMode ? 'max-h-full max-w-full cursor-zoom-in' : 'max-w-none cursor-zoom-out'"
        :style="{
          transform: `rotate(${rotation}deg)`,
          width: !fitMode && baseWidth ? `${Math.round(baseWidth * zoom)}px` : undefined,
        }"
        @load="onImageLoad"
        @dblclick="fitMode ? stepZoom(0.5) : resetFit()"
      />
    </div>

    <div
      v-if="showOcr && ocrPassages.length"
      class="max-h-[34%] shrink-0 space-y-2 overflow-y-auto border-t border-stroke-divider bg-surface-raised px-5 py-3.5"
    >
      <div data-reader-ignore-selection class="text-caption-sm font-semibold text-fg-tertiary select-none">
        {{ t("spaces.inspector.extractedDocumentText") }}
      </div>
      <div class="space-y-2 text-body leading-relaxed text-fg-primary">
        <SpaceTextQuote v-for="(passage, idx) in ocrPassages" :key="idx" :text="passage" :exact="quote" />
      </div>
    </div>
  </div>
</template>
