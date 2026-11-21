<script setup lang="ts">
import { BookOpenText, FileText, Maximize2, Search, ZoomIn, ZoomOut } from "@lucide/vue";
import { computed, shallowRef, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiFindBar from "../../../../shared/ui/UiFindBar.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiSegmentedControl from "../../../../shared/ui/UiSegmentedControl.vue";
import type { KnowledgeLocator } from "../../knowledge/model/schema/chunk";
import type { KnowledgeChunk } from "../../knowledge/model/schema/persisted-chunk";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceFilePayload } from "../../model/nodes/node-payload";
import SpaceDocumentReadingSurface from "./SpaceDocumentReadingSurface.vue";
import { useDocxPreview } from "./use-docx-preview";
import { useDomBlockFind } from "./use-dom-block-find";

const props = defineProps<{
  node: SpaceCanvasNode<SpaceFilePayload>;
  chunks: readonly KnowledgeChunk[];
  loading: boolean;
  locator?: KnowledgeLocator | undefined;
  quote?: string | undefined;
}>();

const { t } = useI18n();
const scrollerRef = useTemplateRef<HTMLDivElement>("scroller");
const hostRef = useTemplateRef<HTMLDivElement>("host");
const styleRef = useTemplateRef<HTMLDivElement>("styleHost");
const viewMode = shallowRef<"word" | "text">("word");

const find = useDomBlockFind(hostRef, {
  enabled: () => viewMode.value === "word",
  selector: "p, td, li, h1, h2, h3, h4",
});
const docx = useDocxPreview({
  scroller: scrollerRef,
  host: hostRef,
  styleHost: styleRef,
  assetPath: () => props.node.data.assetPath,
  active: computed(() => viewMode.value === "word"),
  onRendered: () => {
    if (props.quote?.trim()) find.runFind(props.quote);
  },
});

const modeOptions = computed(() => [
  { value: "word" as const, label: "Word", icon: FileText },
  { value: "text" as const, label: t("spaces.inspector.extractedDocumentText"), icon: BookOpenText },
]);

function onKeyDown(event: KeyboardEvent) {
  if (find.onKeyDown(event) || !(event.metaKey || event.ctrlKey) || viewMode.value !== "word") return;
  const delta = event.key === "=" || event.key === "+" ? 0.1 : event.key === "-" ? -0.1 : 0;
  if (delta) {
    event.preventDefault();
    docx.adjustZoom(delta);
  } else if (event.key === "0") {
    event.preventDefault();
    docx.resetFit();
  }
}

watch(
  () => props.node.id,
  () => {
    viewMode.value = "word";
    find.reset();
  },
);
</script>

<template>
  <div class="flex h-full min-h-0 flex-col bg-surface-canvas outline-none" tabindex="-1" @keydown="onKeyDown">
    <div
      data-reader-ignore-selection
      class="flex h-9 shrink-0 items-center justify-between gap-2 border-b border-stroke-divider bg-surface-panel px-2.5 select-none"
    >
      <UiSegmentedControl
        v-if="chunks.length || node.data.extractedText?.trim() || node.data.previewText?.trim()"
        v-model="viewMode"
        :options="modeOptions"
      />
      <div v-if="viewMode === 'word'" class="ml-auto flex items-center gap-0.5">
        <UiIconButton
          :label="t('spaces.inspector.reader.searchPlaceholder')"
          size="sm"
          tooltip
          :class="find.open.value && 'bg-surface-control text-fg-primary'"
          @click="find.toggle()"
        >
          <Search class="size-3.5" />
        </UiIconButton>
        <span class="mx-0.5 h-3.5 w-px bg-stroke-divider" />
        <UiIconButton
          :label="t('spaces.inspector.pdfReader.zoomOut')"
          size="sm"
          tooltip
          @click="docx.adjustZoom(-0.1)"
        >
          <ZoomOut class="size-3.5" />
        </UiIconButton>
        <button
          type="button"
          class="min-w-10 rounded-sm px-1 py-0.5 text-center text-caption-sm text-fg-secondary tabular-nums hover:bg-surface-control-hover hover:text-fg-primary"
          @click="docx.resetFit()"
        >
          {{ docx.zoomPercent.value }}%
        </button>
        <UiIconButton
          :label="t('spaces.inspector.pdfReader.zoomIn')"
          size="sm"
          tooltip
          @click="docx.adjustZoom(0.1)"
        >
          <ZoomIn class="size-3.5" />
        </UiIconButton>
        <UiIconButton
          :label="t('spaces.inspector.pdfReader.fitWidth')"
          size="sm"
          tooltip
          @click="docx.resetFit()"
        >
          <Maximize2 class="size-3.5" />
        </UiIconButton>
      </div>
    </div>
    <UiFindBar
      v-if="viewMode === 'word' && find.open.value"
      v-model="find.query.value"
      :match-current="find.matched.value.length ? find.matchIndex.value + 1 : 0"
      :match-total="find.matched.value.length"
      @next="find.next()"
      @prev="find.prev()"
      @close="find.reset()"
    />
    <div class="relative min-h-0 flex-1 overflow-hidden">
      <div
        ref="scroller"
        class="absolute inset-0 overflow-auto bg-surface-control/75"
        :class="viewMode === 'word' ? 'visible' : 'pointer-events-none invisible'"
        :inert="viewMode !== 'word'"
      >
        <div ref="styleHost" aria-hidden="true" />
        <p v-if="docx.failure.value" class="p-6 text-center text-caption text-fg-danger" role="alert">
          {{ docx.failure.value }}
        </p>
        <p
          v-else-if="docx.rendering.value"
          data-reader-ignore-selection
          class="p-6 text-center text-caption text-fg-tertiary select-none"
        >
          {{ t("common.loading") }}
        </p>
        <div ref="host" class="word-preview-host px-3 py-3" :style="{ zoom: docx.zoom.value }" />
      </div>
      <SpaceDocumentReadingSurface
        v-if="viewMode === 'text'"
        class="absolute inset-0"
        :node="node"
        :chunks="chunks"
        :loading="loading"
        :locator="locator"
        :quote="quote"
      />
    </div>
  </div>
</template>

<style scoped>
.word-preview-host :deep(.knovum-docx-wrapper) {
  background: transparent;
  padding: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.75rem;
}
.word-preview-host :deep(section.knovum-docx) {
  margin-bottom: 0;
  background: #ffffff;
  color: #1a1a1a;
  border: 1px solid rgb(0 0 0 / 0.08);
  border-radius: 2px;
  box-shadow: 0 1px 3px rgb(0 0 0 / 0.08);
}
</style>
