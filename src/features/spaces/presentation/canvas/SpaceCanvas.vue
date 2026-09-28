<script setup lang="ts">
import { computed, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import { useCanvasToolStore } from "../../canvas/canvas-tool-store";
import type { CanvasSceneInput } from "../../canvas/contracts/canvas-surface";
import {
  currentCanvasEditor,
  updateCanvasDirectManipulationProjection,
  useCanvasEditorSession,
} from "../../canvas/editor-session";
import CanvasSurfaceHost from "../../canvas/host/CanvasSurfaceHost.vue";
import { useNodeOperationGate } from "../../canvas/node-operation-gate";
import { isSpaceNodeOpenable, openSpaceNode } from "../../canvas/selection-actions";
import { useContextDock } from "../../canvas/use-context-dock";
import { createCanvasSelectionActions } from "../../session/actions/canvas-selection-actions";
import { useSpaceStore } from "../../session/space-store";
import { createCanvasContentLayoutCoordinator } from "./canvas-content-layout-coordinator";
import { createCanvasNodeContentPersistence } from "./canvas-node-content-persistence";
import SpaceCanvasNodeContent from "./nodes/SpaceCanvasNodeContent.vue";
import SpaceCanvasProductLayer from "./SpaceCanvasProductLayer.vue";
import { useCanvasContentLayoutSession } from "./use-canvas-content-layout-session";
import { useCanvasNavigationRequests } from "./use-canvas-navigation-requests";
import { useCanvasTextCreation } from "./use-canvas-text-creation";
import { createSpaceCanvasEvents } from "./use-space-canvas-events";
import { useSpaceCanvasImports } from "./use-space-canvas-imports";
import { useSpaceCanvasLifecycle } from "./use-space-canvas-lifecycle";
import { useVisibleCanvasContent } from "./use-visible-canvas-content";

const store = useSpaceStore();
const tools = useCanvasToolStore();
const createTextNode = useCanvasTextCreation();
const editorSession = useCanvasEditorSession();
const operations = useNodeOperationGate();
const dock = useContextDock();
const { t } = useI18n();
const canvasRoot = useTemplateRef<HTMLElement>("canvasRoot");
const selection = createCanvasSelectionActions(store);
const persistNodeContent = createCanvasNodeContentPersistence(store);
const { visibleNodes, visibleRelations } = useVisibleCanvasContent(
  () => store.nodes,
  () => store.relations,
  () => store.relationKindFilter,
  () => store.relationStatusFilter,
);
const sceneInput = computed<CanvasSceneInput>(() => ({
  editSnapshot: store.editSnapshot,
  nodes: store.nodes,
  relations: visibleRelations.value,
}));
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
const lifecycle = useSpaceCanvasLifecycle({
  contentLayout,
  navigation: navigationRequests,
  spaceId,
  store,
  tools,
});
watch(
  () => tools.tool,
  (tool) => {
    if (tool === "section") store.groupScopePath = [];
  },
);
const imports = useSpaceCanvasImports({
  store,
  editorSession,
  dropTarget: () => canvasRoot.value,
  newNoteTitle: t("spaces.editor.note.new"),
  reportError: lifecycle.reportError,
});

async function activateNode(nodeId: string) {
  const node = store.nodes.find((candidate) => candidate.id === nodeId);
  if (!node) return;
  if (!operations.allows(node.data.kind === "note" || node.data.kind === "markdown" ? "edit" : "inspect"))
    return;
  editorSession.selectNodes([nodeId]);
  if (isSpaceNodeOpenable(node)) await openSpaceNode(store, node);
  else dock.openInspect();
}

const canvasEvents = createSpaceCanvasEvents({
  activateNode,
  attachEditor: lifecycle.attachEditor,
  contentLayout,
  createTextNode,
  imports,
  reportError: lifecycle.reportError,
  selection,
  store,
  tools,
  untitledSection: t("spaces.canvas.sections.untitled"),
});
</script>

<template>
  <section ref="canvasRoot" class="relative h-full min-h-0 overflow-hidden bg-surface-canvas select-none">
    <div
      v-if="lifecycle.error.value && !editorSession.activeEditor.value"
      class="pointer-events-none absolute inset-0 z-20 grid place-items-center p-6 text-fg-secondary"
    >
      <p class="max-w-md text-center text-body-lg">{{ lifecycle.error.value }}</p>
    </div>
    <CanvasSurfaceHost
      :key="spaceId"
      :events="canvasEvents"
      :persistence="store.draftSpace?.id === spaceId ? 'draft' : 'persisted'"
      :ensure-persisted="store.ensureActiveSpacePersisted"
      :import-json-canvas="imports.importJsonCanvas"
      :focus-node-ids="store.canvasFocusIds"
      :group-scope-path="store.groupScopePath"
      :interaction-enabled="!editorSession.transitioning.value"
      :tool="tools.tool"
      :space-id="spaceId"
      :scene-input="sceneInput"
      :exchange-relations="store.relations"
      @exchange-ready="lifecycle.attachExchange"
    >
      <template #node-content="{ interactive, layout, node, previewPolicy }">
        <SpaceCanvasNodeContent
          :layout="layout"
          :node="node"
          :preview-policy="previewPolicy"
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
        ...(lifecycle.error.value ? { error: lifecycle.error.value } : {}),
        ...(imports.fileHoverPosition.value ? { fileHoverPosition: imports.fileHoverPosition.value } : {}),
      }"
      @dismiss-error="lifecycle.error.value = undefined"
      @dismiss-capture="store.finishUrlCapture"
      @retry-capture="imports.retryCapture"
    />
  </section>
</template>
