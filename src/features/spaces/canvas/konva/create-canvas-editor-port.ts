import { indexStructure } from "../../model/structure/structure-index";
import {
  structureOperationRoots,
  structureContentNodeIds,
} from "../../organization/structure/structure-selection";
import { captureCanvasOverview, captureCanvasViewport } from "../canvas-overview-capture";
import type { CanvasEditorPort } from "../contract/canvas-editor-port";
import { autoLayoutCanvas } from "../document/canvas-auto-layout";
import { createCanvasSelectionSnapshot } from "../document/canvas-selection-snapshot";
import type { KonvaEditorPortOptions } from "./canvas-editor-port-options";
import { createNodeContentHeightBatch } from "./node-content-height-batch";
import { projectCanvasScene } from "./scene/scene-projection";
import { createCanvasSelectionDeletion } from "./selection/canvas-selection-deletion";
import {
  fitCanvasLayouts,
  selectedCanvasLayouts,
  setCanvasZoom,
  zoomCanvasViewport,
} from "./viewport/canvas-editor-navigation";
import { createStructureNavigation } from "./viewport/structure-navigation";
import { canvasPointFromScreen, screenPointFromCanvas } from "./viewport/viewport-math";

export function createKonvaEditorPort(options: KonvaEditorPortOptions): CanvasEditorPort {
  const selected = () => [...options.selectedNodeIds()];
  const transfer = () => ({ document: options.model.document, itemIds: selected() });
  const deletion = createCanvasSelectionDeletion(options);
  const navigation = createStructureNavigation(options);
  const animated = { ...options, setViewport: options.animateViewport };
  const heights = createNodeContentHeightBatch(options.model, options.nodeById);
  const snapshot = () => ({
    ...createCanvasSelectionSnapshot(
      options.layouts(),
      options.selectedNodeIds(),
      options.relationIds(),
      options.relationBounds(),
    ),
    scopeGroupId: options.scopeGroupId(),
  });
  return {
    cancelInteraction: options.cancelInteraction,
    executeStructureCommand: (command) =>
      command.kind === "delete-subtrees"
        ? deletion(command.ids, [])
        : options.executeStructureCommand(command),
    enterGroup: options.enterGroup,
    exitGroup: options.exitGroup,
    requestContainerRename: options.requestContainerRename,
    copySelection: () => options.onCopyNodes(transfer()),
    cutSelection: () => deletion(selected(), [...options.relationIds()], true),
    pasteSelection: () => options.onPasteNodes(options.pastePoint()),
    duplicateSelection: () => options.onDuplicateNodes(transfer()),
    deleteSelection: () => deletion(selected(), [...options.relationIds()]),
    deleteItems: (ids, relations = []) => deletion(ids, relations),
    nudgeSelection: options.nudgeSelection,
    moveSelectionLayer: (mode) => options.model.moveNodes(options.selectedNodeIds(), mode),
    alignSelection: (mode) => {
      const axis = ["left", "center", "right"].includes(mode) ? "x" : "y";
      const alignment = ["left", "top"].includes(mode)
        ? "start"
        : ["right", "bottom"].includes(mode)
          ? "end"
          : "center";
      return options.executeStructureCommand({ kind: "align-items", ids: selected(), axis, alignment });
    },
    distributeSelection: (mode) =>
      options.executeStructureCommand({
        kind: "distribute-items",
        ids: selected(),
        axis: mode === "horizontal" ? "x" : "y",
      }),
    autoLayoutSelection: (direction) => {
      const tree = indexStructure(options.model.document.structure);
      const roots = structureOperationRoots(tree, selected());
      const representatives = new Map<string, string>();
      for (const root of roots)
        for (const nodeId of structureContentNodeIds(tree, [root.id])) representatives.set(nodeId, root.id);
      const edges = options.relations().flatMap((relation) => {
        const sourceId = representatives.get(relation.sourceId),
          targetId = representatives.get(relation.targetId);
        return sourceId && targetId && sourceId !== targetId && relation.status !== "invalidated"
          ? [{ id: relation.id, sourceId, targetId }]
          : [];
      });
      const ids = new Set(roots.map((root) => root.id));
      return options.runFrameMutation(() =>
        autoLayoutCanvas(
          options.layouts().filter((layout) => ids.has(layout.nodeId)),
          edges,
          direction,
        ),
      );
    },
    fitSection: (id) => options.executeStructureCommand({ kind: "fit-section", id }),
    selectAll: () => options.setSelection(options.selectableNodeIds()),
    clearSelection: () => options.setSelection([]),
    selectNodes: navigation.select,
    reveal: navigation.reveal,
    zoomIn: () => zoomCanvasViewport(animated, 1.2),
    zoomOut: () => zoomCanvasViewport(animated, 1 / 1.2),
    setZoom: (zoom) => setCanvasZoom(animated, zoom),
    navigateViewport: options.animateViewport,
    setViewport: options.setViewport,
    setRelationColor: (id, color) => options.model.setRelationColor(id, color),
    setNodeAppearance: (id, patch) => options.model.setNodeAppearance(id, patch),
    fitNodeContentHeight: heights.fit,
    flushContentHeightFitting: heights.flush,
    setContentHeightFittingPaused: heights.setPaused,
    getViewport: options.viewport,
    getDocument: () => options.model.document,
    observeDocument: (observer) => options.model.observe(observer),
    adoptDocument: (document) => options.model.replace(document),
    zoomToFit: () =>
      fitCanvasLayouts(
        animated,
        projectCanvasScene(options.model.document, options.nodes(), options.relations()).nodes.map(
          (item) => item.layout,
        ),
      ),
    zoomToSelection: () => fitCanvasLayouts(animated, selectedCanvasLayouts(options)),
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
    getSelectionSnapshot: snapshot,
    getSelectionBounds: () => snapshot().bounds,
    screenToCanvasPoint: (point) => {
      const rect = options.host().getBoundingClientRect();
      return canvasPointFromScreen({ x: point.x - rect.left, y: point.y - rect.top }, options.viewport());
    },
    canvasToViewportPoint: (point) => screenPointFromCanvas(point, options.viewport()),
  };
}
