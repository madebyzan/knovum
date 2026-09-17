import { captureCanvasOverview, captureCanvasViewport } from "../canvas-overview-capture";
import type { CanvasEditorPort, CanvasPoint, CanvasRect } from "../contract/canvas-editor-port";
import {
  alignCanvasLayouts,
  distributeCanvasLayouts,
  fitSectionLayout,
} from "../document/canvas-arrangement";
import { autoLayoutCanvas } from "../document/canvas-auto-layout";
import { removeCanvasSelection } from "../document/canvas-content";
import {
  canvasNodePlacements,
  createCanvasSelectionSnapshot,
  movableCanvasNodeIds,
} from "../document/canvas-selection-snapshot";
import { createCanvasPersistenceHistory } from "../persistence/canvas-persistence-history";
import type { KonvaEditorPortOptions } from "./canvas-editor-port-options";
import { canvasAutoLayoutEdges } from "./document/canvas-auto-layout-edges";
import type { CanvasDocumentModel } from "./document/canvas-document-model";
import { createNodeContentHeightBatch } from "./node-content-height-batch";
import { projectCanvasScene } from "./scene/scene-projection";
import {
  fitCanvasLayouts,
  revealCanvasLayouts,
  selectedCanvasLayouts,
  setCanvasZoom,
  zoomCanvasViewport,
} from "./viewport/canvas-editor-navigation";
import { canvasPointFromScreen, screenPointFromCanvas } from "./viewport/viewport-math";

export function createKonvaEditorPort(options: KonvaEditorPortOptions): CanvasEditorPort {
  const selectedPlacements = () => canvasNodePlacements(options.layouts(), options.selectedNodeIds());
  const animatedNavigation = { ...options, setViewport: options.animateViewport };
  const contentHeight = createNodeContentHeightBatch(options.model, options.nodeById);
  return {
    cancelInteraction: options.cancelInteraction,
    copySelection: () => options.onCopyNodes(selectedPlacements()),
    cutSelection: () => deleteSelection(true),
    pasteSelection: () => options.onPasteNodes(options.pastePoint()),
    deleteSelection: () => deleteSelection(false),
    duplicateSelection: () => options.onDuplicateNodes(selectedPlacements()),
    nudgeSelection: (offset) => void options.nudgeSelection(offset).catch(options.onError),
    moveSelectionLayer: (mode) => options.model.moveNodes(options.selectedNodeIds(), mode),
    alignSelection: (mode) => {
      const nodeIds = movableSelectedNodeIds();
      return options.runFrameMutation(() => alignCanvasLayouts(layoutsFor(nodeIds), mode));
    },
    autoLayoutSelection: (direction) => {
      const nodeIds = movableSelectedNodeIds();
      return options.runFrameMutation(() => {
        const targetLayouts = layoutsFor(nodeIds);
        return autoLayoutCanvas(targetLayouts, canvasAutoLayoutEdges(options.relations()), direction);
      });
    },
    distributeSelection: (mode) => {
      const nodeIds = movableSelectedNodeIds();
      return options.runFrameMutation(() => distributeCanvasLayouts(layoutsFor(nodeIds), mode));
    },
    fitSection: (sectionId) => {
      const memberIds = new Set(
        options
          .nodes()
          .filter(({ data }) => data.sectionId === sectionId)
          .map(({ id }) => id),
      );
      return options.runFrameMutation(() => {
        const section = options.layouts().find(({ nodeId }) => nodeId === sectionId);
        if (!section) return [];
        return fitSectionLayout(
          section,
          options.layouts().filter(({ nodeId }) => memberIds.has(nodeId)),
        );
      });
    },
    selectAll: () => options.setSelection(options.selectableNodeIds()),
    zoomIn: () => zoomCanvasViewport(animatedNavigation, 1.2),
    zoomOut: () => zoomCanvasViewport(animatedNavigation, 1 / 1.2),
    setZoom: (zoom) => setCanvasZoom(animatedNavigation, zoom),
    reveal: (ids, mode) => revealCanvasLayouts(animatedNavigation, ids, mode),
    navigateViewport: options.animateViewport,
    setViewport: options.setViewport,
    setRelationColor: (relationId, color) => options.model.setRelationColor(relationId, color),
    setNodeAppearance: (nodeId, appearance) => options.model.setNodeAppearance(nodeId, appearance),
    fitNodeContentHeight: contentHeight.fit,
    flushContentHeightFitting: contentHeight.flush,
    setContentHeightFittingPaused: contentHeight.setPaused,
    getViewport: options.viewport,
    getDocument: () => options.model.document,
    observeDocument: (observer) => options.model.observe(observer),
    adoptDocument: (document) => options.model.applyLocalDocument(document),
    zoomToFit: () => fitCanvasLayouts(animatedNavigation, options.layouts()),
    zoomToSelection: () => fitCanvasLayouts(animatedNavigation, selectedCanvasLayouts(options)),
    clearSelection: () => options.setSelection([]),
    captureSceneViewport: () =>
      captureCanvasViewport(
        projectCanvasScene(options.model.document, options.nodes(), options.relations()),
        options.palette(),
        options.viewport(),
        options.size(),
      ),
    captureCanvasOverview: () =>
      captureCanvasOverview(
        projectCanvasScene(options.model.document, options.nodes(), options.relations()),
        options.palette(),
      ),
    getSelectionSnapshot: () =>
      createCanvasSelectionSnapshot(
        options.layouts(),
        options.selectedNodeIds(),
        options.relationIds(),
        options.relationBounds(),
      ),
    getSelectionBounds: () =>
      createCanvasSelectionSnapshot(
        options.layouts(),
        options.selectedNodeIds(),
        options.relationIds(),
        options.relationBounds(),
      ).bounds,
    selectNodes: options.setSelection,
    screenToCanvasPoint: (point) => {
      const bounds = options.host().getBoundingClientRect();
      return canvasPointFromScreen({ x: point.x - bounds.left, y: point.y - bounds.top }, options.viewport());
    },
    canvasToViewportPoint: (point) => screenPointFromCanvas(point, options.viewport()),
  };

  async function deleteSelection(cut: boolean) {
    const nodeIds = [...options.selectedNodeIds()];
    const relationIds = [...options.relationIds()];
    if (!nodeIds.length && !relationIds.length) return;
    if (cut && !nodeIds.length) return;
    await options.onHistoryTransaction(async () => {
      const before = options.model.document;
      const document = removeCanvasSelection(before, new Set(nodeIds), new Set(relationIds));
      const result = cut
        ? await options.onCutSelection({
            nodeIds,
            nodePlacements: selectedPlacements(),
            relationIds,
            document,
          })
        : await options.onDeleteSelection({ nodeIds, relationIds, document });
      options.model.applyLocalDocument(result.document);
      options.onHistoryEntry(
        createCanvasPersistenceHistory({
          before,
          after: result.document,
          persistence: result.history,
          current: () => options.model.document,
          commit: options.onHistoryTransition,
          adopt: (next) => options.model.applyLocalDocument(next),
        }),
      );
    });
    options.setSelection([]);
  }

  function movableSelectedNodeIds() {
    return movableCanvasNodeIds(options.nodes(), options.selectedNodeIds());
  }

  function layoutsFor(nodeIds: ReadonlySet<string>) {
    return options.layouts().filter(({ nodeId }) => nodeIds.has(nodeId));
  }
}
