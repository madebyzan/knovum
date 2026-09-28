import { createStructureIndex } from "../../model/structure/structure-index";
import type { CanvasEditorPort } from "../contracts/canvas-editor-port";
import { serializeCanvasAppearance } from "../schema/document";
import type { CanvasNodeAppearance } from "../schema/document";
import type { KonvaEditorPortOptions } from "./canvas-editor-port-options";
import { createCanvasEditorSelection } from "./canvas-editor-selection";
import { createNodeContentHeightBatch, type FittedContentHeight } from "./node-content-height-batch";
import { captureCanvasOverview, captureCanvasViewport } from "./overview/capture";
import {
  fitCanvasBounds,
  fitCanvasLayouts,
  revealCanvasTargets,
  setCanvasZoom,
  zoomCanvasViewport,
} from "./viewport/canvas-editor-navigation";
import { canvasPointFromScreen, screenPointFromCanvas } from "./viewport/viewport-math";

export function createKonvaEditorPort(options: KonvaEditorPortOptions): CanvasEditorPort {
  const animatedNavigation = { ...options, setViewport: options.animateViewport };
  const selection = createCanvasEditorSelection(options);
  const report = (operation: Promise<unknown>) => void operation.catch(options.onError);
  let contentHeightTail = Promise.resolve();
  const contentHeight = createNodeContentHeightBatch(
    options.nodeById,
    (id) => options.layouts().find(({ nodeId }) => nodeId === id),
    updateContentHeights,
  );
  return {
    cancelInteraction: options.cancelInteraction,
    copySelection: () => options.onCopyNodes(selection.placements()),
    cutSelection: async () => {
      await options.onCopyNodes(selection.placements());
      await deleteSelection();
    },
    pasteSelection: () => options.onPasteNodes(options.pastePoint()),
    deleteSelection,
    duplicateSelection: async () => {
      const roots = selection.roots();
      if (!roots.length) return;
      const targetParentId = roots.every((root) => root.parentId === roots[0]?.parentId)
        ? (roots[0]?.parentId ?? null)
        : null;
      await options.onExecuteSpaceEdit({
        kind: "structure.clone",
        rootIds: roots.map(({ id }) => id),
        targetParentId,
        offset: { x: 32, y: 32 },
      });
    },
    nudgeSelection: (offset) => void options.nudgeSelection(offset).catch(options.onError),
    moveSelectionLayer: (direction) => report(executeForSelection({ kind: "structure.reorder", direction })),
    alignSelection: (edge) => executeForSelection({ kind: "structure.align", edge }),
    autoLayoutSelection: (direction) =>
      executeForSelection({
        kind: "structure.auto-layout",
        direction: direction === "RIGHT" ? "right" : "down",
        gap: 32,
      }),
    distributeSelection: (axis) => executeForSelection({ kind: "structure.distribute", axis }),
    selectAll: () => options.setSelection(options.selectableNodeIds()),
    zoomIn: () => zoomCanvasViewport(animatedNavigation, 1.2),
    zoomOut: () => zoomCanvasViewport(animatedNavigation, 1 / 1.2),
    setZoom: (zoom) => setCanvasZoom(animatedNavigation, zoom),
    reveal: (ids, mode) => revealCanvasTargets(animatedNavigation, ids, mode, options.structureBounds()),
    navigateViewport: options.animateViewport,
    setViewport: options.setViewport,
    setRelationColor: (relationId, color) =>
      report(
        options.onExecuteSpaceEdit({
          kind: "relation.set-style",
          relationId,
          color: color === "neutral" ? null : color,
        }),
      ),
    setNodeAppearance: (nodeId, patch) => updateAppearance(nodeId, patch),
    fitNodeContentHeight: contentHeight.fit,
    flushContentHeightFitting: contentHeight.flush,
    setContentHeightFittingPaused: contentHeight.setPaused,
    getViewport: options.viewport,
    getDocument: () => options.model.document,
    observeDocument: (observer) => options.model.observe(observer),
    adoptDocument: (document) => options.model.applyLocalDocument(document),
    zoomToFit: () => fitCanvasLayouts(animatedNavigation, options.layouts()),
    zoomToSelection: () => {
      const bounds = selection.bounds();
      if (bounds) fitCanvasBounds(animatedNavigation, bounds);
    },
    clearSelection: selection.clear,
    captureSceneViewport: () =>
      captureCanvasViewport(options.scene(), options.palette(), options.viewport(), options.size()),
    captureCanvasOverview: () => captureCanvasOverview(options.scene(), options.palette()),
    getSelectionSnapshot: selection.snapshot,
    getSelectionBounds: selection.bounds,
    selectNodes: options.setSelection,
    selectStructureNodes: options.setStructureSelection,
    screenToCanvasPoint: (point) => {
      const bounds = options.host().getBoundingClientRect();
      return canvasPointFromScreen({ x: point.x - bounds.left, y: point.y - bounds.top }, options.viewport());
    },
    canvasToViewportPoint: (point) => screenPointFromCanvas(point, options.viewport()),
  };

  async function deleteSelection() {
    const roots = selection.roots();
    const relationIds = [...options.relationIds()];
    if (!roots.length && !relationIds.length) return;
    if (relationIds.length) await options.onDeleteRelations(relationIds);
    else {
      await options.onExecuteSpaceEdit({
        kind: "structure.delete-subtrees",
        rootIds: roots.map(({ id }) => id),
      });
    }
    selection.clear();
  }

  async function executeForSelection(command: AlignOrDistributeCommand) {
    const rootIds = selection.roots().map(({ id }) => id);
    if (rootIds.length) await options.onExecuteSpaceEdit({ ...command, rootIds });
  }

  function updateAppearance(nodeId: string, patch: Partial<CanvasNodeAppearance>) {
    const snapshot = options.snapshot();
    const index = createStructureIndex(snapshot.structure);
    const structureNodeId = index.nodeIdToStructureNodeId.get(nodeId);
    const layout = structureNodeId
      ? snapshot.layouts.find((candidate) => candidate.structureNodeId === structureNodeId)
      : undefined;
    if (!structureNodeId || layout?.kind !== "content")
      throw new Error(`Content layout is unavailable: ${nodeId}`);
    const appearance = serializeCanvasAppearance({ ...layout.appearance, ...patch });
    report(
      options.onExecuteSpaceEdit({
        kind: "content.update-layout",
        structureNodeId,
        appearance: Object.keys(appearance).length ? { kind: "set", value: appearance } : { kind: "clear" },
      }),
    );
  }

  function updateContentHeights(heights: ReadonlyMap<string, FittedContentHeight>) {
    const heightCommit = contentHeightTail.then(async () => {
      const snapshot = options.snapshot();
      const index = createStructureIndex(snapshot.structure);
      const updates = [...heights].flatMap(([nodeId, measurement]) =>
        contentHeightUpdate(nodeId, measurement, snapshot, index),
      );
      if (updates.length) await options.onExecuteSpaceEdit({ kind: "content.update-heights", updates });
      return undefined;
    });
    contentHeightTail = heightCommit.catch(() => undefined);
    report(heightCommit);
  }

  function contentHeightUpdate(
    nodeId: string,
    measurement: FittedContentHeight,
    snapshot: ReturnType<KonvaEditorPortOptions["snapshot"]>,
    index: ReturnType<typeof createStructureIndex>,
  ) {
    const structureNodeId = index.nodeIdToStructureNodeId.get(nodeId);
    const layout = structureNodeId
      ? snapshot.layouts.find((candidate) => candidate.structureNodeId === structureNodeId)
      : undefined;
    if (!structureNodeId || layout?.kind !== "content") return [];
    const matchesSource = [
      (options.nodeById(nodeId)?.data.knowledgeRevision ?? null) === measurement.contentRevision,
      layout.sizingMode === measurement.sizingMode,
      layout.size.width === measurement.measuredWidth,
      (layout.appearance?.typographyScale ?? 1) === measurement.typographyScale,
    ].every(Boolean);
    return matchesSource ? [{ structureNodeId, ...measurement }] : [];
  }
}

type AlignOrDistributeCommand =
  | { kind: "structure.align"; edge: Parameters<CanvasEditorPort["alignSelection"]>[0] }
  | { kind: "structure.distribute"; axis: Parameters<CanvasEditorPort["distributeSelection"]>[0] }
  | { kind: "structure.reorder"; direction: Parameters<CanvasEditorPort["moveSelectionLayer"]>[0] }
  | { kind: "structure.auto-layout"; direction: "right" | "down"; gap: number };
