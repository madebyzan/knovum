<script setup lang="ts">
import { StickyNote } from "@lucide/vue";
import { computed, shallowRef, useTemplateRef, watch } from "vue";

import UiDragGhost from "../../../../shared/ui/UiDragGhost.vue";
import { usePointerDrag } from "../../../../shared/ui/use-pointer-drag";
import { useContextDock } from "../../canvas/use-context-dock";
import type { KnowledgeLocation } from "../../knowledge/model/schema/chunk";
import { useSpaceStore } from "../../session/space-store";
import {
  resolveChunkRangeLocation,
  resolveMediaRangeLocation,
  resolvePdfRangeLocation,
  resolveProseRangeLocation,
  resolveSlideRangeLocation,
  resolveWebsiteRangeLocation,
} from "../reader-location-resolvers";
import { classifyReadableSource, defaultSourceLocation } from "../source-reader-target";
import { useReadingChunks } from "../use-reading-chunks";
import SpaceReaderSelectionBar from "./SpaceReaderSelectionBar.vue";
import SpaceSourceReaderHeader from "./SpaceSourceReaderHeader.vue";
import SpaceSourceReaderSurface from "./SpaceSourceReaderSurface.vue";
import { useReaderCapture } from "./use-reader-capture";
import {
  useReaderSelection,
  type SourceReaderPassage,
  type SurfaceSelectionState,
} from "./use-reader-selection";

const store = useSpaceStore();
const dock = useContextDock();
const body = useTemplateRef<HTMLElement>("body");
const currentPdfPage = shallowRef(1);
const surfaceSelection = shallowRef<SurfaceSelectionState>();
const target = computed(() => store.readingTarget);
const sourceNode = computed(() =>
  target.value ? store.nodes.find((node) => node.id === target.value?.nodeId && !node.hidden) : undefined,
);
const readerKind = computed(() =>
  sourceNode.value ? classifyReadableSource(sourceNode.value.data) : undefined,
);
const targetQuote = computed(() => target.value?.quote ?? target.value?.locator?.quote?.exact);

watch(
  [target, sourceNode, readerKind],
  ([currentTarget, currentNode, currentKind]) => {
    if (currentTarget && (!currentNode || !currentKind)) store.closeSourceReader();
  },
  { flush: "post" },
);

watch(
  () => target.value?.nonce,
  (nonce) => {
    // Collapse the redundant Details inspector when entering Read & Organize mode so Reader + Canvas have room.
    if (nonce !== undefined && store.rightSidebarView === "inspect") dock.closeDock();
  },
  { immediate: true },
);

const readingChunks = useReadingChunks(() => store.activeSpaceId, sourceNode, readerKind);

function resolveLocation(range: Range, host: HTMLElement): KnowledgeLocation | undefined {
  const kind = readerKind.value;
  const node = sourceNode.value;
  if (!kind || !node) return undefined;
  if (kind === "pdf") return resolvePdfRangeLocation(range, currentPdfPage.value);
  if (kind === "slides") return resolveSlideRangeLocation(range);
  if (kind === "website") return resolveWebsiteRangeLocation(range);
  if (kind === "media") return resolveMediaRangeLocation(range, host);
  if (kind === "markdown" || kind === "note") return resolveProseRangeLocation(range, kind);
  return resolveChunkRangeLocation(range, readingChunks.chunks.value, defaultSourceLocation(node.data));
}

const selection = useReaderSelection(body, resolveLocation);
const capture = useReaderCapture(sourceNode);
const activePassage = computed(() => surfaceSelection.value?.passage ?? selection.passage.value);
const activeAnchor = computed(() => surfaceSelection.value?.anchor ?? selection.anchor.value);

function clearAllSelections() {
  selection.clear();
  surfaceSelection.value?.clear?.();
  surfaceSelection.value = undefined;
}

watch(
  () => [sourceNode.value?.id, target.value?.nonce] as const,
  () => {
    clearAllSelections();
    currentPdfPage.value = 1;
  },
);

const drag = usePointerDrag({
  targetAt: capture.canvasPointAt,
  tap: clearAllSelections,
  drop: (passage: SourceReaderPassage, point) => {
    clearAllSelections();
    return capture.createNote(passage, point);
  },
});

function onMouseDown(event: MouseEvent) {
  const passage = selection.passage.value;
  if (passage && selection.covers({ x: event.clientX, y: event.clientY })) drag.start(event, passage);
}

async function runAction(action: (passage: SourceReaderPassage) => unknown) {
  const passage = activePassage.value;
  clearAllSelections();
  if (passage) await action(passage);
}
</script>

<template>
  <aside
    v-if="sourceNode && readerKind"
    data-source-reader
    data-bare-key-scope
    :data-reader-kind="readerKind"
    class="flex h-full min-h-0 min-w-0 flex-col border-r border-stroke-divider bg-surface-panel"
  >
    <SpaceSourceReaderHeader :node="sourceNode" @close="store.closeSourceReader()" />
    <div
      ref="body"
      data-reader-body
      class="relative min-h-0 flex-1 overflow-hidden select-text"
      @mousedown="onMouseDown"
      @contextmenu="selection.onContextMenu"
    >
      <SpaceReaderSelectionBar
        v-if="activeAnchor && !drag.drag.value"
        class="absolute z-30 -translate-x-1/2"
        :class="activeAnchor.placement === 'top' && '-translate-y-full'"
        :style="{ left: `${activeAnchor.left}px`, top: `${activeAnchor.top}px` }"
        :can-save="capture.canSave.value"
        :selected-append-target="capture.selectedAppendTarget.value"
        :existing-note-targets="capture.existingNoteTargets.value"
        @create-note="runAction((passage) => capture.createNote(passage))"
        @create-evidence="runAction((passage) => capture.createEvidence(passage))"
        @append="(targetId) => runAction((passage) => capture.appendToNode(passage, targetId))"
        @ask="runAction((passage) => capture.askAboutPassage(passage))"
        @copy="runAction((passage) => capture.copyPassage(passage))"
      />
      <SpaceSourceReaderSurface
        :node="sourceNode"
        :kind="readerKind"
        :chunks="readingChunks.chunks.value"
        :loading="readingChunks.loading.value"
        :recognized-pdf-pages="readingChunks.recognizedPdfPages.value"
        :recognized-text="readingChunks.recognizedText"
        :locator="target?.locator"
        :quote="targetQuote"
        @pdf-page-change="currentPdfPage = $event"
        @surface-selection="surfaceSelection = $event"
      />
    </div>
    <UiDragGhost
      v-if="drag.drag.value"
      :x="drag.drag.value.x"
      :y="drag.drag.value.y"
      :droppable="drag.drag.value.target !== undefined"
    >
      <StickyNote class="size-3.5 shrink-0" />
      <span class="truncate">{{ drag.drag.value.item.text }}</span>
    </UiDragGhost>
  </aside>
</template>
