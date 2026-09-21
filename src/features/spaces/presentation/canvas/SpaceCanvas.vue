<script setup lang="ts">
import { computed, onBeforeUnmount, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import type { CanvasExchangePort } from "../../canvas-exchange/canvas-exchange-port";
import { registerCanvasExchangePort } from "../../canvas-exchange/canvas-exchange-session";
import { createCanvasSelectionActions } from "../../canvas/canvas-selection-actions";
import { useCanvasToolStore } from "../../canvas/canvas-tool-store";
import type { CanvasHostEvents } from "../../canvas/contract/canvas-editor";
import type { CanvasEditorPort } from "../../canvas/contract/canvas-editor-port";
import {
  currentCanvasEditor,
  registerCanvasEditor,
  updateCanvasDirectManipulationProjection,
  updateCanvasSelectionProjection,
  updateCanvasViewportProjection,
  useCanvasEditorSession,
} from "../../canvas/editor-session";
import CanvasSurfaceHost from "../../canvas/host/CanvasSurfaceHost.vue";
import { isSpaceNodeOpenable, openSpaceNode } from "../../canvas/selection-actions";
import { useContextDock } from "../../canvas/use-context-dock";
import { useSpaceStore } from "../../session/space-store";
import { createCanvasContentLayoutCoordinator } from "./canvas-content-layout-coordinator";
import { createCanvasNodeContentPersistence } from "./canvas-node-content-persistence";
import SpaceCanvasNodeContent from "./nodes/SpaceCanvasNodeContent.vue";
import SpaceCanvasProductLayer from "./SpaceCanvasProductLayer.vue";
import { useCanvasContentLayoutSession } from "./use-canvas-content-layout-session";
import { useCanvasNavigationRequests } from "./use-canvas-navigation-requests";
import { useCanvasTextCreation } from "./use-canvas-text-creation";
import { useSpaceCanvasImports } from "./use-space-canvas-imports";
import { useVisibleCanvasContent } from "./use-visible-canvas-content";

const store = useSpaceStore();
const tools = useCanvasToolStore();
const createTextNode = useCanvasTextCreation();
const editorSession = useCanvasEditorSession();
const dock = useContextDock();
const { t } = useI18n();
let detachEditor: (() => void) | undefined;
let detachOverviewObserver: (() => void) | undefined;
let detachExchange: (() => void) | undefined;
const canvasError = shallowRef<string>();
const selection = createCanvasSelectionActions(store);
const persistNodeContent = createCanvasNodeContentPersistence(store);
const { visibleNodes, visibleRelations } = useVisibleCanvasContent(
  () => store.nodes,
  () => store.relations,
  () => store.relationKindFilter,
  () => store.relationStatusFilter,
);
const navigationRequests = useCanvasNavigationRequests(store, editorSession);
const contentLayout = createCanvasContentLayoutCoordinator({
  directManipulation: () => editorSession.directManipulation.value,
  editor: currentCanvasEditor,
  projectDirectManipulation: updateCanvasDirectManipulationProjection,
  projectMutationPending: store.setCanvasMutationPending,
});
useCanvasContentLayoutSession(contentLayout);
function canvasToViewportPoint(point: { x: number; y: number }) {
  const editor = editorSession.activeEditor.value;
  if (!editor) throw new Error("The canvas editor must be mounted before projecting a canvas point");
  return editor.canvasToViewportPoint(point);
}
const spaceId = computed(() => {
  if (!store.activeSpaceId) throw new Error("An active space is required to mount its canvas");
  return store.activeSpaceId;
});

function reportCanvasError(error: unknown) {
  canvasError.value = error instanceof Error ? error.message : String(error);
}

function attachEditor(editor: CanvasEditorPort) {
  detachOverviewObserver?.();
  detachEditor?.();
  detachEditor = registerCanvasEditor(editor);
  let previous = editor.getDocument();
  detachOverviewObserver = editor.observeDocument((document) => {
    if (document.nodes !== previous.nodes || document.relationStyles !== previous.relationStyles)
      store.markCanvasOverviewDirty();
    previous = document;
  });
  contentLayout.sync();
  canvasError.value = undefined;
  navigationRequests.consumePending();
}

function attachExchange(exchange: CanvasExchangePort) {
  detachExchange?.();
  detachExchange = registerCanvasExchangePort(exchange);
}

watch(spaceId, () => {
  tools.resetTool();
  detachOverviewObserver?.();
  detachOverviewObserver = undefined;
  detachEditor?.();
  detachEditor = undefined;
  detachExchange?.();
  detachExchange = undefined;
  canvasError.value = undefined;
});
const imports = useSpaceCanvasImports({
  store,
  editorSession,
  newNoteTitle: t("spaces.editor.newNote"),
  reportError: reportCanvasError,
});

async function activateNode(nodeId: string) {
  const node = store.nodes.find((candidate) => candidate.id === nodeId);
  if (!node) return;
  editorSession.selectNodes([nodeId]);
  if (isSpaceNodeOpenable(node)) await openSpaceNode(store, node);
  else dock.openInspect();
}

const canvasEvents: CanvasHostEvents = {
  onCreateTextNode: createTextNode,
  onCommitGesture: store.commitGesture,
  onCopyNodes: selection.copyNodes,
  onCreateRelation: async (sourceNodeId, targetNodeId) => {
    await store.createRelation(sourceNodeId, targetNodeId, "related");
  },
  onCutSelection: selection.cutSelection,
  onDeleteSelection: selection.deleteSelection,
  onDuplicateNodes: selection.duplicateNodes,
  onInsertText: imports.insertText,
  onInsertUrl: imports.insertUrl,
  onImportFiles: imports.importFiles,
  onNodeActivate: activateNode,
  onPasteNodes: imports.pasteNodes,
  onError: reportCanvasError,
  onHistoryEntry: store.recordCanvasHistory,
  onHistoryTransaction: store.recordHistoryMutation,
  onDirectManipulationChange: contentLayout.updateDirectManipulation,
  onMutationPendingChange: contentLayout.updateMutationPending,
  onReady: attachEditor,
  onSelectionChange: updateCanvasSelectionProjection,
  onViewportChange: updateCanvasViewportProjection,
};

onBeforeUnmount(() => {
  tools.resetTool();
  detachOverviewObserver?.();
  detachEditor?.();
  detachExchange?.();
});
</script>

<template>
  <section class="relative h-full min-h-0 overflow-hidden bg-canvas select-none">
    <div
      v-if="canvasError && !editorSession.activeEditor.value"
      class="pointer-events-none absolute inset-0 z-20 grid place-items-center p-6 text-foreground-secondary"
    >
      <p class="max-w-md text-center text-sm">{{ canvasError }}</p>
    </div>
    <CanvasSurfaceHost
      :key="spaceId"
      :events="canvasEvents"
      :persistence="store.draftSpace?.id === spaceId ? 'draft' : 'persisted'"
      :ensure-persisted="store.ensureActiveSpacePersisted"
      :import-json-canvas="imports.importJsonCanvas"
      :focus-node-ids="store.canvasFocusIds"
      :interaction-enabled="!editorSession.transitioning.value"
      :tool="tools.tool"
      :space-id="spaceId"
      :nodes="store.nodes"
      :relations="visibleRelations"
      @exchange-ready="attachExchange"
    >
      <template #node-content="{ interactive, layout, node, detailLevel }">
        <SpaceCanvasNodeContent
          :layout="layout"
          :node="node"
          :detail-level="detailLevel"
          :interactive="interactive"
          :persist="persistNodeContent"
          :fit-content-height="contentLayout.fit"
        />
      </template>
    </CanvasSurfaceHost>
    <SpaceCanvasProductLayer
      v-if="editorSession.activeEditor.value && !editorSession.transitioning.value"
      :captures="store.urlCaptures"
      :canvas-to-viewport-point="canvasToViewportPoint"
      :empty="visibleNodes.length === 0 && store.urlCaptures.length === 0"
      :file-hover="imports.fileHover.value"
      :space-id="spaceId"
      v-bind="{
        ...(canvasError ? { error: canvasError } : {}),
        ...(imports.fileHoverPosition.value ? { fileHoverPosition: imports.fileHoverPosition.value } : {}),
      }"
      @dismiss-error="canvasError = undefined"
      @dismiss-capture="store.finishUrlCapture"
      @retry-capture="imports.retryCapture"
    />
  </section>
</template>
