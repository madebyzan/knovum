<script setup lang="ts">
import { computed, provide, useTemplateRef, watch } from "vue";

import { overlayScopeVisibleKey } from "../../../../shared/contracts/overlay-scope-visibility";
import { createCanvasSelectionActions } from "../../session/actions/canvas-selection-actions";
import { useSpaceStore } from "../../session/space-store";
import { useCanvasToolStore } from "../canvas-tool-store";
import type { CanvasSceneInput } from "../contracts/canvas-surface";
import {
  currentCanvasEditor,
  updateCanvasDirectManipulationProjection,
  getCanvasEditorSession,
} from "../editor-session";
import CanvasSurfaceHost from "../host/CanvasSurfaceHost.vue";
import { isSpaceNodeOpenable, openSpaceNode } from "../selection-actions";
import { useContextDock } from "../use-context-dock";
import { useNodeOperationGate } from "../use-node-operation-gate";
import { createCanvasContentLayoutCoordinator } from "./canvas-content-layout-coordinator";
import { createCanvasNodeContentPersistence } from "./canvas-node-content-persistence";
import SpaceCanvasNodeContent from "./nodes/SpaceCanvasNodeContent.vue";
import { createSpaceCanvasEvents } from "./space-canvas-events";
import SpaceCanvasProductLayer from "./SpaceCanvasProductLayer.vue";
import { useCanvasContentLayoutSession } from "./use-canvas-content-layout-session";
import { useCanvasNavigationRequests } from "./use-canvas-navigation-requests";
import { useCanvasTextCreation } from "./use-canvas-text-creation";
import { useSpaceCanvasImports } from "./use-space-canvas-imports";
import { useSpaceCanvasLifecycle } from "./use-space-canvas-lifecycle";
import { useVisibleCanvasContent } from "./use-visible-canvas-content";

const props = defineProps<{ renderingEnabled: boolean }>();
provide(
  overlayScopeVisibleKey,
  computed(() => props.renderingEnabled),
);
const store = useSpaceStore();
const tools = useCanvasToolStore();
const createTextNode = useCanvasTextCreation();
const editorSession = getCanvasEditorSession();
const operations = useNodeOperationGate();
const canCommitCanvasGestures = computed(() => operations.allowsGestureCommit());
const dock = useContextDock();
const canvasRoot = useTemplateRef<HTMLElement>("canvasRoot");
const selection = createCanvasSelectionActions();
const persistNodeContent = createCanvasNodeContentPersistence();
const { visibleRelations } = useVisibleCanvasContent({
  nodes: () => store.nodes,
  relations: () => store.relations,
  relationKinds: () => store.relationKindFilter,
  relationStatuses: () => store.relationStatusFilter,
});
const sceneInput = computed<CanvasSceneInput>(() => ({
  editSnapshot: store.editSnapshot,
  nodes: store.nodes,
  relations: visibleRelations.value,
}));
const hasCanvasObjects = computed(() => store.editSnapshot.structure.length > 0);
const initialEmpty = computed(
  () => !hasCanvasObjects.value && (tools.tool === "select" || tools.tool === "pan"),
);
const navigationRequests = useCanvasNavigationRequests();
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
});
watch(
  () => tools.tool,
  (tool) => {
    if (tool === "section") store.groupScopePath = [];
  },
);
const imports = useSpaceCanvasImports(canvasRoot);
const canvasError = computed(() => lifecycle.error.value ?? imports.error.value);

function dismissError() {
  lifecycle.error.value = undefined;
  imports.error.value = undefined;
}

async function activateNode(nodeId: string) {
  const node = store.nodes.find((candidate) => candidate.id === nodeId);
  if (!node) return;
  if (!operations.allows(node.data.kind === "note" || node.data.kind === "markdown" ? "edit" : "inspect"))
    return;
  editorSession.selectNodes([nodeId]);
  if (isSpaceNodeOpenable(node)) await openSpaceNode(node);
  else dock.openInspect();
}

const canvasEvents = createSpaceCanvasEvents({
  activateNode,
  attachEditor: lifecycle.attachEditor,
  contentLayout,
  createTextNode,
  imports,
  operations,
  reportError: lifecycle.reportError,
  selection,
});
</script>

<template>
  <section
    v-show="renderingEnabled"
    :inert="!renderingEnabled"
    ref="canvasRoot"
    class="relative h-full min-h-0 overflow-hidden bg-surface-canvas select-none"
  >
    <div
      v-if="lifecycle.error.value && !editorSession.activeEditor.value"
      class="pointer-events-none absolute inset-0 z-20 grid place-items-center p-6 text-fg-secondary"
    >
      <p class="max-w-md text-center text-body-lg">{{ lifecycle.error.value }}</p>
    </div>
    <CanvasSurfaceHost
      :key="spaceId"
      :rendering-enabled="renderingEnabled"
      :events="canvasEvents"
      :persistence="store.spacePersistence(spaceId)"
      :ensure-persisted="store.ensureActiveSpacePersisted"
      :import-json-canvas="store.importJsonCanvas"
      :focus-node-ids="store.canvasFocusIds"
      :group-scope-path="store.groupScopePath"
      :interaction-enabled="renderingEnabled && !editorSession.transitioning.value"
      :navigation-enabled="hasCanvasObjects"
      :can-commit-canvas-gestures="canCommitCanvasGestures"
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
      v-if="renderingEnabled && editorSession.activeEditor.value && !editorSession.transitioning.value"
      :canvas-to-viewport-point="canvasToViewportPoint"
      :empty="initialEmpty"
      :navigation-enabled="hasCanvasObjects"
      :file-hover="imports.fileHover.value"
      v-bind="{
        ...(canvasError ? { error: canvasError } : {}),
        ...(imports.fileHoverPosition.value ? { fileHoverPosition: imports.fileHoverPosition.value } : {}),
      }"
      @dismiss-error="dismissError"
    />
  </section>
</template>
