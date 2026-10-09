import { compact } from "es-toolkit";
import { readonly, shallowRef } from "vue";

import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import { snapshotSpaceSessionParticipants } from "../workspace/session-participants";
import type {
  CanvasEditorPort,
  CanvasSelectionSnapshot,
  CanvasViewport,
} from "./contracts/canvas-editor-port";

const activeEditor = shallowRef<CanvasEditorPort>();
const viewport = shallowRef<CanvasViewport>();
const emptySelection: CanvasSelectionSnapshot = {
  nodeIds: [],
  structureNodeIds: [],
  nodePlacements: [],
  members: [],
  relationIds: [],
};
const selection = shallowRef<CanvasSelectionSnapshot>(emptySelection);
const directManipulation = shallowRef(false);
const transitioning = shallowRef(false);
const activeTextSelectionNodeId = shallowRef<string>();
const activeTextEditorNodeId = shallowRef<string>();
let transitionTail = Promise.resolve();
let pendingTransitions = 0;
let suspendedEditor: CanvasEditorPort | undefined;
let suspendedParticipants = snapshotSpaceSessionParticipants();

const editorSession = {
  activeEditor,
  activeTextSelectionNodeId: readonly(activeTextSelectionNodeId),
  activeTextEditorNodeId: readonly(activeTextEditorNodeId),
  directManipulation: readonly(directManipulation),
  transitioning: readonly(transitioning),
  selection: readonly(selection),
  viewport,
  clearSelection: () => activeEditor.value?.clearSelection(),
  reveal: (nodeIds: readonly string[], mode: "ensure-visible" | "focus") =>
    activeEditor.value?.reveal(nodeIds, mode),
  selectNodes: (nodeIds: readonly string[]) => activeEditor.value?.selectNodes(nodeIds),
  selectStructureNodes: (structureNodeIds: readonly string[]) =>
    activeEditor.value?.selectStructureNodes(structureNodeIds),
};

export function getCanvasEditorSession() {
  return editorSession;
}

export function currentCanvasEditor() {
  return activeEditor.value ?? suspendedEditor;
}

export function registerCanvasEditor(editor: CanvasEditorPort) {
  if (transitioning.value) suspendedEditor = editor;
  else activeEditor.value = editor;
  viewport.value = editor.getViewport();
  selection.value = editor.getSelectionSnapshot();
  return () => {
    if (suspendedEditor === editor) suspendedEditor = undefined;
    if (activeEditor.value !== editor) return;
    activeEditor.value = undefined;
    viewport.value = undefined;
    selection.value = emptySelection;
    directManipulation.value = false;
    activeTextSelectionNodeId.value = undefined;
    activeTextEditorNodeId.value = undefined;
  };
}

export function updateCanvasSelectionProjection(value: CanvasSelectionSnapshot) {
  selection.value = value;
}

export function updateCanvasViewportProjection(value: CanvasViewport) {
  viewport.value = value;
}

export function updateCanvasDirectManipulationProjection(active: boolean) {
  directManipulation.value = active;
}

export function updateCanvasTextSelectionProjection(nodeId: string, active: boolean) {
  if (active) {
    activeTextSelectionNodeId.value = nodeId;
  } else if (activeTextSelectionNodeId.value === nodeId) {
    activeTextSelectionNodeId.value = undefined;
  }
}

export function updateCanvasTextEditorProjection(nodeId: string, active: boolean) {
  if (active) activeTextEditorNodeId.value = nodeId;
  else if (activeTextEditorNodeId.value === nodeId) activeTextEditorNodeId.value = undefined;
}

export function runCanvasSessionTransition<T>(transition: () => Promise<T>) {
  if (pendingTransitions === 0) {
    suspendedEditor = activeEditor.value;
    suspendedEditor?.cancelInteraction();
    if (activeEditor.value === suspendedEditor) activeEditor.value = undefined;
    suspendedParticipants = snapshotSpaceSessionParticipants();
    for (const participant of suspendedParticipants) participant.suspend?.();
  }
  pendingTransitions += 1;
  transitioning.value = true;
  const operation = transitionTail.then(transition);
  transitionTail = operation.then(
    () => undefined,
    () => undefined,
  );
  return operation.finally(() => {
    pendingTransitions -= 1;
    transitioning.value = pendingTransitions > 0;
    if (pendingTransitions === 0 && !activeEditor.value) activeEditor.value = suspendedEditor;
    if (pendingTransitions === 0) {
      for (const participant of suspendedParticipants) participant.resume?.();
      suspendedParticipants = [];
      suspendedEditor = undefined;
    }
  });
}

export function selectedCanvasNodesInOrder(
  nodes: readonly SpaceCanvasNode[],
  selectedNodeIds: readonly string[],
) {
  const byId = new Map(nodes.map((node) => [node.id, node]));
  return compact(selectedNodeIds.map((id) => byId.get(id)));
}
