<script setup lang="ts">
import { watchDebounced } from "@vueuse/core";
import { computed, ref, shallowRef, useTemplateRef, watch } from "vue";

import UiFindBar from "../../../../shared/ui/UiFindBar.vue";
import { selectAndRevealSpaceNode } from "../../canvas/selection-actions";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { useSpaceStore } from "../../session/space-store";
import type { PdfTextPage } from "./pdf-editor-params";
import SpacePdfCanvasViewport from "./SpacePdfCanvasViewport.vue";
import SpacePdfMarkupBar from "./SpacePdfMarkupBar.vue";
import SpacePdfOcrDrawer from "./SpacePdfOcrDrawer.vue";
import SpacePdfSidebar from "./SpacePdfSidebar.vue";
import SpacePdfTextView from "./SpacePdfTextView.vue";
import SpacePdfToolbar from "./SpacePdfToolbar.vue";
import { usePdfMarkup } from "./use-pdf-markup";
import { usePdfProvenanceOverlay } from "./use-pdf-provenance-overlay";
import { usePdfReaderLifecycle } from "./use-pdf-reader-lifecycle";
import { usePdfReaderShortcuts } from "./use-pdf-reader-shortcuts";
import { usePdfSourceLinks, type PdfProvenanceLink } from "./use-pdf-source-links";

const props = defineProps<{
  node?: SpaceCanvasNode | undefined;
  path: string;
  page?: number | undefined;
  passage?: string | undefined;
  textPages?: readonly PdfTextPage[] | undefined;
  recognizedPages?: ReadonlyMap<number, readonly string[]> | undefined;
}>();
const emit = defineEmits<{ pageChange: [page: number] }>();

const store = useSpaceStore();
const canvasViewport = useTemplateRef<InstanceType<typeof SpacePdfCanvasViewport>>("canvasViewport");
const container = computed(() => canvasViewport.value?.container ?? null);
const viewer = computed(() => canvasViewport.value?.viewer ?? null);
const viewMode = shallowRef<"pdf" | "text">("pdf");
const sidebarOpen = shallowRef(false);
const areaMode = shallowRef(false);
const findOpen = shallowRef(false);
const findQuery = ref("");

const sourceLinks = usePdfSourceLinks(() => props.node?.id);
const provenanceOverlay = usePdfProvenanceOverlay(
  viewer,
  sourceLinks.links,
  computed(() => props.passage),
  (targetNodeId) => selectAndRevealSpaceNode(targetNodeId, "focus"),
);
const lifecycle = usePdfReaderLifecycle({
  container,
  viewer,
  getNode: () => props.node,
  getPath: () => props.path,
  getInitialPage: () => props.page,
  getPassage: () => props.passage,
  onPageChange: (page) => emit("pageChange", page),
  onPageRendered: (page) => provenanceOverlay.syncPage(page),
});
const markup = usePdfMarkup(lifecycle.session);
const onKeyDown = usePdfReaderShortcuts({
  container,
  session: lifecycle.session,
  viewMode,
  areaMode,
  findOpen,
  modified: lifecycle.modified,
  markupOpen: markup.open,
  markupTool: markup.tool,
  selectMarkupTool: markup.selectTool,
  toggleMarkup: markup.toggle,
  savePdf: lifecycle.savePdf,
});

const pageRecognizedPassages = computed(() => props.recognizedPages?.get(lifecycle.currentPage.value) ?? []);

function toggleArea() {
  areaMode.value = !areaMode.value;
  if (areaMode.value && markup.open.value) markup.toggle();
}
function toggleMarkup() {
  if (!markup.open.value) areaMode.value = false;
  markup.toggle();
}
function toggleFind() {
  findOpen.value = !findOpen.value;
  if (findOpen.value) return;
  findQuery.value = "";
  lifecycle.session.value?.clearFind();
}
function jumpLink(link: PdfProvenanceLink) {
  lifecycle.session.value?.navigate(link.page, link.targetKind === "image" ? undefined : link.quote);
}

watch(lifecycle.session, () => {
  markup.reset();
  areaMode.value = false;
});
watchDebounced(findQuery, (q) => findOpen.value && lifecycle.session.value?.find(q), { debounce: 120 });
</script>

<template>
  <div class="flex h-full min-h-0 flex-col bg-surface-canvas outline-none" tabindex="-1" @keydown="onKeyDown">
    <SpacePdfToolbar
      v-model:view-mode="viewMode"
      :session="lifecycle.session.value"
      :has-text-view="Boolean(textPages?.length)"
      :sidebar-open="sidebarOpen"
      :find-open="findOpen"
      :area-mode="areaMode"
      :markup-open="markup.open.value"
      :can-markup="Boolean(node && store.activeSpaceId)"
      :modified="lifecycle.modified.value"
      :current-page="lifecycle.currentPage.value"
      :page-count="lifecycle.pageCount.value"
      :scale-percent="lifecycle.scalePercent.value"
      :spread-mode="lifecycle.spreadMode.value"
      @toggle-sidebar="sidebarOpen = !sidebarOpen"
      @toggle-find="toggleFind"
      @toggle-area="toggleArea"
      @toggle-markup="toggleMarkup"
      @set-spread="lifecycle.setSpread"
    />
    <SpacePdfMarkupBar
      v-if="viewMode === 'pdf' && markup.open.value"
      :markup="markup"
      :session="lifecycle.session.value"
      :modified="lifecycle.modified.value"
      :saving="lifecycle.saving.value"
      @discard="lifecycle.reload()"
      @save="lifecycle.savePdf()"
      @done="toggleMarkup"
    />
    <UiFindBar
      v-if="viewMode === 'pdf' && findOpen"
      v-model="findQuery"
      :match-current="lifecycle.findMatches.value.current"
      :match-total="lifecycle.findMatches.value.total"
      @next="lifecycle.session.value?.find(findQuery, 'next')"
      @prev="lifecycle.session.value?.find(findQuery, 'prev')"
      @close="toggleFind"
    />
    <div class="relative min-h-0 flex-1 overflow-hidden">
      <div
        class="absolute inset-0 flex flex-col overflow-hidden"
        :class="viewMode === 'pdf' ? 'visible' : 'pointer-events-none invisible'"
        :inert="viewMode !== 'pdf'"
      >
        <p v-if="lifecycle.failure.value" class="p-6 text-center text-caption text-fg-danger" role="alert">
          {{ lifecycle.failure.value }}
        </p>
        <div class="flex min-h-0 flex-1 overflow-hidden">
          <SpacePdfSidebar
            v-if="sidebarOpen && lifecycle.pageCount.value"
            :session="lifecycle.session.value"
            :page-count="lifecycle.pageCount.value"
            :current-page="lifecycle.currentPage.value"
            :outline="lifecycle.outlineEntries.value"
            :links="sourceLinks.links.value"
            :highlighted-pages="sourceLinks.highlightedPages.value"
            @go-to-page="lifecycle.session.value?.goToPage($event)"
            @jump-link="jumpLink"
            @reveal-canvas="selectAndRevealSpaceNode($event, 'focus')"
          />
          <SpacePdfCanvasViewport
            ref="canvasViewport"
            :area-mode="areaMode"
            :markup-open="markup.open.value"
            @area-capture="(page, rect) => ((areaMode = false), void lifecycle.captureArea(page, rect))"
          />
        </div>
        <SpacePdfOcrDrawer
          v-if="pageRecognizedPassages.length"
          :page="lifecycle.currentPage.value"
          :passages="pageRecognizedPassages"
        />
      </div>
      <SpacePdfTextView
        v-if="viewMode === 'text' && textPages?.length"
        class="absolute inset-0"
        :pages="textPages"
      />
    </div>
  </div>
</template>
