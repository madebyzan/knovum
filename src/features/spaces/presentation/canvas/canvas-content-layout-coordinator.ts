import { nextTick } from "vue";

import type {
  CanvasContentHeightMeasurement,
  CanvasEditorPort,
} from "../../canvas/contracts/canvas-editor-port";

interface CanvasContentLayoutCoordinatorOptions {
  directManipulation(): boolean;
  editor(): CanvasEditorPort | undefined;
  projectDirectManipulation(active: boolean): void;
  projectMutationPending(pending: boolean): void;
}

export function createCanvasContentLayoutCoordinator(options: CanvasContentLayoutCoordinatorOptions) {
  let mutationPending = false;
  const pendingMeasurementByNodeId = new Map<string, CanvasContentHeightMeasurement>();
  const measurementSources = new Set<() => void>();

  function sync() {
    const editor = options.editor();
    if (!editor) return;
    editor.setContentHeightFittingPaused(options.directManipulation() || mutationPending);
    for (const [nodeId, measurement] of pendingMeasurementByNodeId) {
      editor.fitNodeContentHeight(nodeId, measurement);
    }
    pendingMeasurementByNodeId.clear();
  }

  function fit(nodeId: string, measurement: CanvasContentHeightMeasurement) {
    const editor = options.editor();
    if (editor) editor.fitNodeContentHeight(nodeId, measurement);
    else pendingMeasurementByNodeId.set(nodeId, measurement);
  }

  function updateDirectManipulation(active: boolean) {
    options.projectDirectManipulation(active);
    sync();
  }

  function updateMutationPending(pending: boolean) {
    mutationPending = pending;
    options.projectMutationPending(pending);
    sync();
  }

  async function settle() {
    await nextTick();
    for (const measure of measurementSources) measure();
    const editor = options.editor();
    if (editor && !editor.flushContentHeightFitting()) {
      throw new Error("Canvas content layout cannot settle during direct manipulation");
    }
  }

  const measurements = {
    register(measure: () => void) {
      measurementSources.add(measure);
      return () => measurementSources.delete(measure);
    },
  };

  return { fit, measurements, settle, sync, updateDirectManipulation, updateMutationPending };
}

export type CanvasContentLayoutCoordinator = ReturnType<typeof createCanvasContentLayoutCoordinator>;
