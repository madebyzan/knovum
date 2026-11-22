<script setup lang="ts">
import { FilePlus2, StickyNote } from "@lucide/vue";
import { computed, shallowRef, useTemplateRef, watch } from "vue";

import UiDragGhost from "../../../../shared/ui/UiDragGhost.vue";
import { usePointerDrag } from "../../../../shared/ui/use-pointer-drag";
import type { KnowledgeLocation } from "../../knowledge/model/schema/chunk";
import {
  resolveChunkRangeLocation,
  resolveMediaRangeLocation,
  resolvePdfRangeLocation,
  resolveProseRangeLocation,
  resolveSlideRangeLocation,
  resolveWebsiteRangeLocation,
} from "../reader-location-resolvers";
import { defaultSourceLocation } from "../source-reader-target";
import { useReadingChunks } from "../use-reading-chunks";
import SpaceReaderInfoSurface from "./SpaceReaderInfoSurface.vue";
import SpaceReaderSelectionBar from "./SpaceReaderSelectionBar.vue";
import SpaceSourceReaderHeader from "./SpaceSourceReaderHeader.vue";
import SpaceSourceReaderSurface from "./SpaceSourceReaderSurface.vue";
import { useReaderCapture, type ReaderDropTarget } from "./use-reader-capture";
import {
  useReaderSelection,
  type SourceReaderPassage,
  type SurfaceSelectionState,
} from "./use-reader-selection";
import { useSourceReaderState } from "./use-source-reader-state";

const readerState = useSourceReaderState();
const { store, target, sourceNode, readerKind, activePane, readMounted, targetQuote, canBack, canForward } =
  readerState;
const body = useTemplateRef<HTMLElement>("body");
const currentPdfPage = shallowRef(1);
const headerMenuOpen = shallowRef(false);
const surfaceSelection = shallowRef<SurfaceSelectionState>();

const readingChunks = useReadingChunks(
  () => store.activeSpaceId,
  sourceNode,
  computed(() => (readMounted.value ? readerKind.value : undefined)),
);

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

watch([() => sourceNode.value?.id, () => target.value?.nonce, activePane], () => {
  clearAllSelections();
  currentPdfPage.value = 1;
});

const drag = usePointerDrag<SourceReaderPassage, ReaderDropTarget>({
  targetAt: capture.dropTargetAt,
  tap: clearAllSelections,
  drop: (passage, dropTarget) => {
    clearAllSelections();
    return dropTarget.kind === "append"
      ? capture.appendToNode(passage, dropTarget.nodeId)
      : capture.createNote(passage, dropTarget.point);
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
    v-if="sourceNode"
    data-source-reader
    data-bare-key-scope
    :data-reader-kind="readerKind ?? 'info'"
    :data-reader-pane="activePane"
    class="flex h-full min-h-0 min-w-0 flex-col border-r border-(--knovum-window-border-color) bg-surface-panel"
  >
    <SpaceSourceReaderHeader
      v-model:menu-open="headerMenuOpen"
      :node="sourceNode"
      :pane="activePane"
      :readable="Boolean(readerKind)"
      :can-back="canBack"
      :can-forward="canForward"
      :follows-selection="store.readingFollowsSelection"
      :locator="target?.locator"
      @update:pane="store.setReadingPane"
      @step-history="store.stepReadingHistory"
      @toggle-follow="store.readingFollowsSelection = !store.readingFollowsSelection"
      @close="store.closeSourceReader()"
    />
    <div
      ref="body"
      data-reader-body
      class="relative min-h-0 flex-1 overflow-hidden select-text"
      @mousedown="onMouseDown"
      @contextmenu="selection.onContextMenu"
    >
      <SpaceReaderSelectionBar
        v-if="activePane === 'read' && activeAnchor && !drag.drag.value"
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
      <div
        v-if="readerKind && readMounted"
        class="absolute inset-0 isolate z-0 flex flex-col overflow-hidden"
        :class="activePane !== 'read' && 'pointer-events-none invisible'"
        :inert="activePane !== 'read'"
      >
        <SpaceSourceReaderSurface
          :node="sourceNode"
          :kind="readerKind"
          :chunks="readingChunks.chunks.value"
          :loading="readingChunks.loading.value"
          :recognized-pdf-pages="readingChunks.recognizedPdfPages.value"
          :recognized-text="readingChunks.recognizedText"
          :locator="target?.locator"
          :locate-nonce="target?.nonce"
          :quote="targetQuote"
          :visible="activePane === 'read' && !headerMenuOpen"
          @pdf-page-change="currentPdfPage = $event"
          @surface-selection="surfaceSelection = $event"
        />
      </div>
      <SpaceReaderInfoSurface v-if="activePane === 'info'" class="absolute inset-0 z-10" :node="sourceNode" />
    </div>
    <UiDragGhost
      v-if="drag.drag.value"
      :x="drag.drag.value.x"
      :y="drag.drag.value.y"
      :droppable="drag.drag.value.target !== undefined"
    >
      <FilePlus2 v-if="drag.drag.value.target?.kind === 'append'" class="size-3.5 shrink-0" />
      <StickyNote v-else class="size-3.5 shrink-0" />
      <span class="truncate">{{
        drag.drag.value.target?.kind === "append" ? drag.drag.value.target.title : drag.drag.value.item.text
      }}</span>
    </UiDragGhost>
  </aside>
</template>
