"""Reviewed operation routing and aggregate gesture integration for PR 14."""
from pathlib import Path
import re
ROOT=Path('.')
files={}
def read(path): return files[path] if path in files else (ROOT/path).read_text()
def write(path,text): files[path]=text.strip()+'\n'

write('src/features/spaces/canvas/konva/interaction/aggregate-preview.ts', '''
import type { SpaceEditorDocument, CanvasNodeLayoutV1 } from "../../contract/canvas-document";
import { composeStructureTransforms, invertStructureTransform, normalizeStructureRotation, transformStructurePoint } from "../../document/structure-transform";
import { indexStructure, structureSubtree } from "../../../model/structure/structure-index";
import type { CanvasGestureUpdate } from "./canvas-gesture-update";
/** Changes only the visible projection during a Group transform; no command/schema round trip per frame. */
export function expandAggregatePreview(document: SpaceEditorDocument, layouts: readonly CanvasNodeLayoutV1[], updates: readonly CanvasGestureUpdate[]) {
  const tree = indexStructure(document.structure);
  const originals = new Map(layouts.map((layout) => [layout.nodeId, layout]));
  const expanded = new Map(updates.map((update) => [update.nodeId, update]));
  for (const update of updates) {
    if (tree.items.get(update.nodeId)?.kind !== "group") continue;
    const original = originals.get(update.nodeId)!;
    const scale = update.frame.width / original.frame.width;
    const transform = composeStructureTransforms({ x: update.frame.x, y: update.frame.y, rotation: update.frame.rotation, scale },
      invertStructureTransform({ x: original.frame.x, y: original.frame.y, rotation: original.frame.rotation, scale: 1 }));
    for (const item of structureSubtree(tree, [update.nodeId])) {
      if (item.id === update.nodeId) continue;
      const layout = originals.get(item.id)!;
      expanded.set(item.id, { nodeId: item.id, contentScale: (layout.contentScale ?? 1) * scale,
        frame: { ...transformStructurePoint(transform, layout.frame), width: layout.frame.width * scale, height: layout.frame.height * scale, rotation: normalizeStructureRotation(layout.frame.rotation + transform.rotation) } });
    }
  }
  return [...expanded.values()];
}
''')
write('src/features/spaces/canvas/konva/interaction/node-gesture-controller.ts', '''
import type Konva from "konva";
import type { CanvasDisplayNode } from "../../contract/canvas-display-node";
import type { CanvasNodeFrameUpdate, CanvasNodeLayoutV1, SpaceEditorDocument } from "../../contract/canvas-document";
import type { CanvasGestureUpdate } from "./canvas-gesture-update";
import { captureCanvasNodeRefs } from "./canvas-node-access";
import { captureCanvasNodeDragState, restoreCanvasNodeFrames } from "./canvas-node-drag-state";
import { expandAggregatePreview } from "./aggregate-preview";
import type { CanvasGestureFeedback, CanvasSnapGuide } from "./canvas-snapping";
import { captureCanvasTransformTargets, projectCanvasTransformUpdates } from "./canvas-transform-projection";
import { snapCanvasTranslation, type CanvasTranslationGestureState } from "./canvas-translation-snapping";
import { projectResizeFeedback } from "./proportional-resize-feedback";
interface NodeGestureControllerOptions {
  document(): SpaceEditorDocument; layouts(): readonly CanvasNodeLayoutV1[]; nodes(): readonly CanvasDisplayNode[];
  previewFrames(updates: readonly CanvasGestureUpdate[]): void; selectedNodeIds(): ReadonlySet<string>;
  stage(): Konva.Stage | undefined; updateFeedback(feedback: CanvasGestureFeedback | undefined): void; zoom(): number;
}
export class NodeGestureController {
  private drag: CanvasTranslationGestureState | undefined;
  private dragNodes = new Map<string, Konva.Node>();
  private resizeGuides: readonly CanvasSnapGuide[] = [];
  private transforming = false;
  private transformLayouts: readonly CanvasNodeLayoutV1[] = [];
  private transformTargets: ReturnType<typeof captureCanvasTransformTargets> = [];
  private proportionalAnchor: string | undefined;
  constructor(private readonly options: NodeGestureControllerOptions) {}
  readonly beginDrag = (primaryId: string) => {
    this.resizeGuides = []; this.options.updateFeedback(undefined);
    const captured = captureCanvasNodeDragState(this.options.layouts(), this.options.document(), this.options.selectedNodeIds(), primaryId, this.options.stage());
    if (!captured) { captureCanvasNodeRefs(this.options.stage(), [primaryId]).get(primaryId)?.stopDrag(); return false; }
    this.drag = captured.state; this.dragNodes = captured.nodeRefs; return true;
  };
  readonly moveDrag = (primaryId: string) => {
    if (!this.drag || this.drag.primaryId !== primaryId) return;
    const primary = this.dragNodes.get(primaryId); if (!primary) return;
    const proposed = { x: primary.x() - this.drag.origin.x, y: primary.y() - this.drag.origin.y };
    const snapped = snapCanvasTranslation([...this.drag.frames.values()], this.drag.snapIndex, proposed, 6 / this.options.zoom());
    const updates = [...this.drag.frames].map(([nodeId, frame]) => ({ nodeId, frame: { ...frame, x: frame.x + snapped.x, y: frame.y + snapped.y } }));
    for (const update of updates) this.dragNodes.get(update.nodeId)?.position(update.frame);
    this.options.previewFrames(updates); this.options.updateFeedback({ guides: snapped.guides });
  };
  endDrag(primaryId: string): CanvasGestureUpdate[] {
    if (!this.drag || this.drag.primaryId !== primaryId) return [];
    const current = this.drag; const primary = this.dragNodes.get(primaryId);
    this.drag = undefined; this.dragNodes.clear(); this.options.updateFeedback(undefined);
    if (!primary) return [];
    const delta = { x: primary.x() - current.origin.x, y: primary.y() - current.origin.y };
    return [...current.frames].map(([nodeId, frame]) => ({ nodeId, frame: { ...frame, x: frame.x + delta.x, y: frame.y + delta.y } }));
  }
  beginTransform() {
    this.transformLayouts = this.options.layouts();
    this.transformTargets = captureCanvasTransformTargets(this.transformLayouts, this.options.nodes(), this.options.selectedNodeIds(), this.options.stage());
    this.transforming = true;
  }
  readonly previewTransform = (anchor?: string) => {
    this.proportionalAnchor = anchor;
    const updates = this.transformedLayouts();
    const expanded = expandAggregatePreview(this.options.document(), this.transformLayouts, updates);
    const roots = new Set(updates.map((update) => update.nodeId));
    const refs = captureCanvasNodeRefs(this.options.stage(), expanded.map((update) => update.nodeId));
    const before = new Map(this.transformLayouts.map((layout) => [layout.nodeId, layout.frame]));
    for (const update of expanded) {
      if (roots.has(update.nodeId)) continue;
      const node = refs.get(update.nodeId), frame = before.get(update.nodeId)!;
      node?.position(update.frame); node?.rotation(update.frame.rotation);
      node?.scale({ x: update.frame.width / frame.width, y: update.frame.height / frame.height });
    }
    this.options.previewFrames(expanded);
    const frame = updates[0]?.frame;
    this.options.updateFeedback(frame ? projectResizeFeedback(frame, this.resizeGuides, this.options.zoom(), anchor) : undefined);
  };
  endTransform() {
    const updates = this.transformedLayouts();
    this.transforming = false; this.proportionalAnchor = undefined; this.transformTargets = [];
    this.options.updateFeedback(undefined); return updates;
  }
  panDrag(delta: { x: number; y: number }) {
    if (!this.drag) return;
    const node = this.dragNodes.get(this.drag.primaryId); if (!node) return;
    node.position({ x: node.x() + delta.x, y: node.y() + delta.y }); this.moveDrag(this.drag.primaryId);
  }
  readonly updateResizeGuides = (guides: readonly CanvasSnapGuide[]) => { this.resizeGuides = guides; };
  cancelActiveGesture() {
    if (!this.drag && !this.transforming) return false;
    const frames = this.drag ? [...this.drag.frames].map(([nodeId, frame]) => ({ nodeId, frame })) : this.transformLayouts;
    this.restoreFrames(frames); this.drag = undefined; this.dragNodes.clear(); this.transforming = false;
    this.transformTargets = []; this.proportionalAnchor = undefined; this.resizeGuides = [];
    this.options.previewFrames([]); this.options.updateFeedback(undefined); return true;
  }
  restoreFrames(updates: readonly CanvasNodeFrameUpdate[], nodes = captureCanvasNodeRefs(this.options.stage(), updates.map((update) => update.nodeId))) { restoreCanvasNodeFrames(updates, nodes); }
  private transformedLayouts() { return projectCanvasTransformUpdates({ proportionalAnchor: this.proportionalAnchor, targets: this.transformTargets }); }
}
''')
write('src/features/spaces/organization/structure/section-drop-intent.ts', '''
import type { StructurePoint } from "../../canvas/contract/structure-layout";
import { indexStructureSnapshot } from "./structure-context";
import type { StructureSnapshot } from "./structure-snapshot-contract";
import { assertStructureEditable, structureOperationRoots } from "./structure-selection";
export function resolveSectionDrop(snapshot: StructureSnapshot, ids: readonly string[], point: StructurePoint, previous: string | null | undefined, zoom: number): string | null | undefined {
  const context = indexStructureSnapshot(snapshot);
  const roots = structureOperationRoots(context.tree, ids);
  assertStructureEditable(context.tree, roots.map((item) => item.id));
  if (roots.some((item) => item.kind === "section" || item.parentId !== null && context.tree.items.get(item.parentId)?.kind === "group")) return undefined;
  const candidates = (context.tree.children.get(null) ?? []).filter((item) => item.kind === "section" && !item.locked).toSorted((a, b) => b.order - a.order);
  const contains = (id: string, margin: number) => {
    const layout = context.layouts.get(id)!;
    return layout.kind === "section" && !layout.collapsed && point.x >= layout.position.x - margin && point.x <= layout.position.x + layout.size.width + margin && point.y >= layout.position.y - margin && point.y <= layout.position.y + layout.size.height + margin;
  };
  const direct = candidates.find((item) => contains(item.id, 0));
  if (direct) return direct.id;
  if (previous && candidates.some((item) => item.id === previous) && contains(previous, 8 / zoom)) return previous;
  return null;
}
''')
write('src/features/spaces/canvas/konva/interaction/canvas-gesture-session.ts', '''
import type Konva from "konva";
import { nextTick, shallowRef } from "vue";
import type { CanvasDisplayNode } from "../../contract/canvas-display-node";
import type { CanvasNodeFrameUpdate, CanvasNodeLayoutV1, SpaceEditorDocument } from "../../contract/canvas-document";
import type { CanvasSurfaceEvents } from "../../contract/canvas-editor";
import type { CanvasPoint } from "../../contract/canvas-editor-port";
import type { StructureCommand } from "../../../organization/structure/commands/contract";
import { planStructureCommand } from "../../../organization/structure/commands/plan";
import { resolveSectionDrop } from "../../../organization/structure/section-drop-intent";
import { projectEditorLayouts } from "../../document/editor-layout-projection";
import { replayEditorTransition } from "../../document/editor-history-transition";
import type { CanvasDocumentModel } from "../document/canvas-document-model";
import { applyCanvasGestureUpdates } from "../document/canvas-document-mutations";
import type { CanvasGestureUpdate } from "./canvas-gesture-update";
import type { CanvasGestureFeedback } from "./canvas-snapping";
import { NodeGestureController } from "./node-gesture-controller";
interface Options {
  document: CanvasDocumentModel; events: CanvasSurfaceEvents;
  layouts(): readonly CanvasNodeLayoutV1[]; nodes(): readonly CanvasDisplayNode[];
  previewFrames(updates: readonly CanvasGestureUpdate[]): void; selectedNodeIds(): ReadonlySet<string>;
  setSelection(ids: readonly string[]): void; stage(): Konva.Stage | undefined; zoom(): number;
}
export type CanvasDirectManipulation = "drag" | "transform";
export function createCanvasGestureSession(options: Options) {
  const busy = shallowRef(false), directManipulation = shallowRef<CanvasDirectManipulation>();
  const feedback = shallowRef<CanvasGestureFeedback>(), dropTarget = shallowRef<string | null>();
  let origin: SpaceEditorDocument | undefined; let ids: readonly string[] = [];
  const controller = new NodeGestureController({ ...options, document: () => options.document.document, previewFrames: preview, updateFeedback: (value) => { feedback.value = value; } });
  function beginDirectManipulation(kind: CanvasDirectManipulation) {
    origin = options.document.document; ids = [...options.selectedNodeIds()]; directManipulation.value = kind;
    if (kind === "transform") controller.beginTransform();
  }
  function preview(updates: readonly CanvasGestureUpdate[]) {
    options.previewFrames(updates);
    if (!origin || !ids.length || directManipulation.value !== "drag") return;
    const point = options.stage()?.getRelativePointerPosition();
    if (point) dropTarget.value = resolveSectionDrop(origin, ids, point, dropTarget.value, options.zoom());
  }
  async function finish(gesture: { kind: "drag"; nodeId: string } | { kind: "transform" }) {
    const before = origin; if (!before) return;
    const selected = ids;
    const target = dropTarget.value;
    const updates = gesture.kind === "drag" ? controller.endDrag(gesture.nodeId) : controller.endTransform();
    await tracked(async () => {
      try {
        let after: SpaceEditorDocument;
        if (gesture.kind === "drag") {
          const start = projectEditorLayouts(before).find((layout) => layout.nodeId === gesture.nodeId)?.frame;
          const frame = updates.find((update) => update.nodeId === gesture.nodeId)?.frame;
          if (!start || !frame) return;
          const delta = { x: frame.x - start.x, y: frame.y - start.y };
          const command: StructureCommand = target === undefined ? { kind: "move-items", ids: [...selected], delta } : { kind: "drop-items", ids: [...selected], parentId: target, delta };
          after = { ...before, ...planStructureCommand(before, command).after };
        } else after = applyCanvasGestureUpdates(before, updates);
        await options.document.mutate((current) => replayEditorTransition(current, before, after));
      } finally {
        await nextTick(); controller.restoreFrames(options.layouts());
        origin = undefined; ids = []; dropTarget.value = undefined; directManipulation.value = undefined;
        feedback.value = undefined; options.previewFrames([]);
      }
    });
  }
  async function runStructureCommand(command: StructureCommand) {
    return tracked(async () => { const selection = await options.document.execute(command, [...options.selectedNodeIds()]); await nextTick(); options.setSelection(selection); });
  }
  async function runFrameMutation(calculate: () => readonly CanvasNodeFrameUpdate[] | Promise<readonly CanvasNodeFrameUpdate[]>) {
    const before = options.document.document;
    const updates = await calculate();
    const layouts = new Map(projectEditorLayouts(before).map((layout) => [layout.nodeId, layout.frame]));
    let after = before;
    for (const update of updates) {
      const frame = layouts.get(update.nodeId); if (!frame) throw new Error("Arrangement target disappeared");
      after = { ...after, ...planStructureCommand(after, { kind: "move-items", ids: [update.nodeId], delta: { x: update.frame.x - frame.x, y: update.frame.y - frame.y } }).after };
    }
    await tracked(() => options.document.mutate((current) => replayEditorTransition(current, before, after)));
  }
  function cancelActiveGesture() { const cancelled = controller.cancelActiveGesture(); if (!busy.value) { origin = undefined; ids = []; dropTarget.value = undefined; directManipulation.value = undefined; } return cancelled; }
  async function tracked(work: () => Promise<void>) { busy.value = true; options.events.onMutationPendingChange(true); try { await work(); } finally { busy.value = false; options.events.onMutationPendingChange(false); } }
  const nudge = (delta: CanvasPoint) => runStructureCommand({ kind: "move-items", ids: [...options.selectedNodeIds()], delta });
  return { busy, controller, directManipulation, feedback, dropTarget, beginDirectManipulation, finish, runStructureCommand, runFrameMutation, cancelActiveGesture, nudge };
}
''')
write('src/features/spaces/canvas/konva/selection/canvas-selection-deletion.ts', '''
import { indexStructure } from "../../../model/structure/structure-index";
import { structureContentNodeIds } from "../../../organization/structure/structure-selection";
import { removeCanvasSelection } from "../../document/canvas-content";
import { createCanvasPersistenceHistory } from "../../persistence/canvas-persistence-history";
import type { KonvaEditorPortOptions } from "../canvas-editor-port-options";
export function createCanvasSelectionDeletion(options: KonvaEditorPortOptions) {
  return async (ids: readonly string[], relationIds: readonly string[], cut = false) => {
    if (!ids.length && !relationIds.length || cut && !ids.length) return;
    const before = options.model.document;
    const nodeIds = structureContentNodeIds(indexStructure(before.structure), ids);
    const content = new Set(nodeIds), selectedRelations = new Set(relationIds);
    for (const relation of options.relations()) if (content.has(relation.sourceId) || content.has(relation.targetId)) selectedRelations.add(relation.id);
    const document = removeCanvasSelection(before, new Set(ids), selectedRelations);
    await options.onHistoryTransaction(async () => {
      const request = { baseDocument: before, document, nodeIds, relationIds: [...selectedRelations] };
      const result = await (cut ? options.onCutSelection({ ...request, transfer: { document: before, itemIds: ids } }) : options.onDeleteSelection(request));
      options.model.replace(result.document);
      options.onHistoryEntry(createCanvasPersistenceHistory({ before, after: result.document, persistence: result.history,
        current: () => options.model.document, commit: options.onHistoryTransition, adopt: (next) => options.model.replace(next) }));
    });
    options.setSelection([]);
  };
}
''')
write('src/features/spaces/canvas/konva/viewport/structure-navigation.ts', '''
import { nextTick } from "vue";
import { indexStructure, structureAncestors } from "../../../model/structure/structure-index";
import type { KonvaEditorPortOptions } from "../canvas-editor-port-options";
import { revealCanvasLayouts } from "./canvas-editor-navigation";
export function createStructureNavigation(options: KonvaEditorPortOptions) {
  function select(ids: readonly string[]) { options.setEditingScope(ids); options.setSelection(ids); }
  async function revealItems(ids: readonly string[], mode: "ensure-visible" | "focus") {
    if (!ids.length) { select([]); return; }
    const before = options.model.document;
    const tree = indexStructure(before.structure);
    const existing = ids.filter((id) => tree.items.has(id));
    const sections = new Set(existing.flatMap((id) => structureAncestors(tree, id).filter((item) => item.kind === "section").map((item) => item.id)));
    if (sections.size) await options.model.mutate((document) => ({ ...document, layout: { version: 1, items: document.layout.items.map((item) => item.kind === "section" && sections.has(item.itemId) ? { ...item, collapsed: false } : item) } }), false);
    options.setEditingScope(existing); await nextTick();
    revealCanvasLayouts({ ...options, setViewport: options.animateViewport }, existing, mode);
  }
  return { select, reveal: (ids: readonly string[], mode: "ensure-visible" | "focus") => { void revealItems(ids, mode).catch(options.onError); } };
}
''')
write('src/features/spaces/canvas/konva/create-canvas-editor-port.ts', '''
import { captureCanvasOverview, captureCanvasViewport } from "../canvas-overview-capture";
import type { CanvasEditorPort } from "../contract/canvas-editor-port";
import { autoLayoutCanvas } from "../document/canvas-auto-layout";
import { createCanvasSelectionSnapshot } from "../document/canvas-selection-snapshot";
import { indexStructure } from "../../model/structure/structure-index";
import { structureOperationRoots, structureContentNodeIds } from "../../organization/structure/structure-selection";
import type { KonvaEditorPortOptions } from "./canvas-editor-port-options";
import { createNodeContentHeightBatch } from "./node-content-height-batch";
import { projectCanvasScene } from "./scene/scene-projection";
import { createCanvasSelectionDeletion } from "./selection/canvas-selection-deletion";
import { createStructureNavigation } from "./viewport/structure-navigation";
import { fitCanvasLayouts, selectedCanvasLayouts, setCanvasZoom, zoomCanvasViewport } from "./viewport/canvas-editor-navigation";
import { canvasPointFromScreen, screenPointFromCanvas } from "./viewport/viewport-math";

export function createKonvaEditorPort(options: KonvaEditorPortOptions): CanvasEditorPort {
  const selected = () => [...options.selectedNodeIds()];
  const transfer = () => ({ document: options.model.document, itemIds: selected() });
  const deletion = createCanvasSelectionDeletion(options);
  const navigation = createStructureNavigation(options);
  const animated = { ...options, setViewport: options.animateViewport };
  const heights = createNodeContentHeightBatch(options.model, options.nodeById);
  const snapshot = () => ({ ...createCanvasSelectionSnapshot(options.layouts(), options.selectedNodeIds(), options.relationIds(), options.relationBounds()), scopeGroupId: options.scopeGroupId() });
  return {
    cancelInteraction: options.cancelInteraction,
    executeStructureCommand: (command) => command.kind === "delete-subtrees" ? deletion(command.ids, []) : options.executeStructureCommand(command),
    enterGroup: options.enterGroup, exitGroup: options.exitGroup, requestContainerRename: options.requestContainerRename,
    copySelection: () => options.onCopyNodes(transfer()), cutSelection: () => deletion(selected(), [...options.relationIds()], true),
    pasteSelection: () => options.onPasteNodes(options.pastePoint()), duplicateSelection: () => options.onDuplicateNodes(transfer()),
    deleteSelection: () => deletion(selected(), [...options.relationIds()]), deleteItems: (ids, relations = []) => deletion(ids, relations),
    nudgeSelection: options.nudgeSelection, moveSelectionLayer: (mode) => options.model.moveNodes(options.selectedNodeIds(), mode),
    alignSelection: (mode) => {
      const axis = ["left", "center", "right"].includes(mode) ? "x" : "y";
      const alignment = ["left", "top"].includes(mode) ? "start" : ["right", "bottom"].includes(mode) ? "end" : "center";
      return options.executeStructureCommand({ kind: "align-items", ids: selected(), axis, alignment });
    },
    distributeSelection: (mode) => options.executeStructureCommand({ kind: "distribute-items", ids: selected(), axis: mode === "horizontal" ? "x" : "y" }),
    autoLayoutSelection: (direction) => {
      const tree = indexStructure(options.model.document.structure);
      const roots = structureOperationRoots(tree, selected());
      const representatives = new Map<string, string>();
      for (const root of roots) for (const nodeId of structureContentNodeIds(tree, [root.id])) representatives.set(nodeId, root.id);
      const edges = options.relations().flatMap((relation) => {
        const sourceId = representatives.get(relation.sourceId), targetId = representatives.get(relation.targetId);
        return sourceId && targetId && sourceId !== targetId && relation.status !== "invalidated" ? [{ id: relation.id, sourceId, targetId }] : [];
      });
      const ids = new Set(roots.map((root) => root.id));
      return options.runFrameMutation(() => autoLayoutCanvas(options.layouts().filter((layout) => ids.has(layout.nodeId)), edges, direction));
    },
    fitSection: (id) => options.executeStructureCommand({ kind: "fit-section", id }),
    selectAll: () => options.setSelection(options.selectableNodeIds()), clearSelection: () => options.setSelection([]),
    selectNodes: navigation.select, reveal: navigation.reveal,
    zoomIn: () => zoomCanvasViewport(animated, 1.2), zoomOut: () => zoomCanvasViewport(animated, 1 / 1.2),
    setZoom: (zoom) => setCanvasZoom(animated, zoom), navigateViewport: options.animateViewport, setViewport: options.setViewport,
    setRelationColor: (id, color) => options.model.setRelationColor(id, color), setNodeAppearance: (id, patch) => options.model.setNodeAppearance(id, patch),
    fitNodeContentHeight: heights.fit, flushContentHeightFitting: heights.flush, setContentHeightFittingPaused: heights.setPaused,
    getViewport: options.viewport, getDocument: () => options.model.document, observeDocument: (observer) => options.model.observe(observer),
    adoptDocument: (document) => options.model.replace(document),
    zoomToFit: () => fitCanvasLayouts(animated, projectCanvasScene(options.model.document, options.nodes(), options.relations()).nodes.map((item) => item.layout)),
    zoomToSelection: () => fitCanvasLayouts(animated, selectedCanvasLayouts(options)),
    captureSceneViewport: () => captureCanvasViewport(projectCanvasScene(options.model.document, options.nodes(), options.relations()), options.palette(), options.viewport(), options.size()),
    captureCanvasOverview: () => captureCanvasOverview(projectCanvasScene(options.model.document, options.nodes(), options.relations()), options.palette()),
    getSelectionSnapshot: snapshot, getSelectionBounds: () => snapshot().bounds,
    screenToCanvasPoint: (point) => { const rect = options.host().getBoundingClientRect(); return canvasPointFromScreen({ x: point.x - rect.left, y: point.y - rect.top }, options.viewport()); },
    canvasToViewportPoint: (point) => screenPointFromCanvas(point, options.viewport()),
  };
}
''')
name='src/features/spaces/canvas/contract/canvas-editor.ts';text=read(name)
text=text.replace('export interface CanvasCutSelectionRequest extends CanvasDeleteSelectionRequest {\n  nodePlacements: readonly CanvasNodePlacement[];\n}', 'export interface CanvasSelectionTransfer { document: SpaceEditorDocument; itemIds: readonly string[] }\nexport interface CanvasCutSelectionRequest extends CanvasDeleteSelectionRequest { transfer: CanvasSelectionTransfer }')
text=text.replace('onCopyNodes(nodes: readonly CanvasNodePlacement[])', 'onCopyNodes(transfer: CanvasSelectionTransfer)').replace('onDuplicateNodes(nodes: readonly CanvasNodePlacement[])', 'onDuplicateNodes(transfer: CanvasSelectionTransfer)')
write(name,text)
# Persisted topology cannot be recreated by content watchers. Creation is handled by Rust use cases.
write('src/features/spaces/canvas/host/use-canvas-document-session.ts', '''
import { shallowRef } from "vue";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { emptySpaceEditorDocument, type SpaceEditorDocument } from "../contract/canvas-document";
import { registerSpaceSessionParticipant } from "../editor-session";
import { acquireCanvasPersistenceLease, registerCanvasMutationTarget, registerCanvasTransactionTarget } from "../persistence/canvas-persistence-coordinator";
import { canvasStateRepository } from "../persistence/canvas-state-repository";
import { CanvasSnapshotSaveQueue } from "../persistence/snapshot-save-queue";
import { CanvasDocumentWorkspace } from "./canvas-document-workspace";
interface Options { nodes(): readonly SpaceCanvasNode[]; reportError(error: unknown): void; spaceId(): string }
export function useCanvasDocumentSession(options: Options) {
  const document = shallowRef<SpaceEditorDocument>(); const surfaceKey = shallowRef(0);
  let workspace: CanvasDocumentWorkspace | undefined;
  let lease: ReturnType<typeof acquireCanvasPersistenceLease> | undefined;
  let detachTargets: (() => void)[] = []; let generation = 0;
  const detachSession = registerSpaceSessionParticipant({ phase: "persistence", commit: async () => { await workspace?.flush(); } });
  function commitTransaction<T extends { document: SpaceEditorDocument; revision: number }>(commit: (baseRevision: number) => Promise<T>) {
    if (!workspace) return Promise.reject(new Error("The Space editor is not ready"));
    return workspace.commit(commit);
  }
  async function start() {
    const expected = ++generation, spaceId = options.spaceId();
    const old = workspace; detachTargets.forEach((detach) => detach()); detachTargets = [];
    lease?.release(release(old)); workspace = undefined; document.value = undefined;
    lease = acquireCanvasPersistenceLease(spaceId); await lease.wait();
    if (generation !== expected) return;
    const stored = await canvasStateRepository.load(spaceId);
    if (generation !== expected) return;
    const initial = stored?.document ?? emptySpaceEditorDocument();
    const queue = new CanvasSnapshotSaveQueue({ spaceId, initialState: stored, persistence: canvasStateRepository, reportError: options.reportError });
    const next = new CanvasDocumentWorkspace(initial, queue, { isActive: () => generation === expected && workspace === next, publish: (value) => { document.value = value; } });
    workspace = next; document.value = initial;
    detachTargets = [registerCanvasTransactionTarget(spaceId, (commit) => next.commit((revision) => commit(next.document, revision))),
      registerCanvasMutationTarget(spaceId, async (mutation) => {
        await next.commit(async (baseRevision) => { const updated = mutation(next.document); return { document: updated, revision: await canvasStateRepository.save({ spaceId, baseRevision, document: updated }) }; });
      })];
    surfaceKey.value += 1;
  }
  async function release(previous: CanvasDocumentWorkspace | undefined) { try { await previous?.flush(); } catch (error) { options.reportError(error); } }
  function dispose() { generation += 1; detachTargets.forEach((detach) => detach()); detachTargets = []; lease?.release(release(workspace)); lease = undefined; workspace = undefined; detachSession(); }
  return { document, surfaceKey, start, dispose, commitTransaction, documentChanged: (next: SpaceEditorDocument) => workspace?.change(next), synchronizeNodes: (_nodes: readonly SpaceCanvasNode[]) => undefined };
}
''')
name='src/features/spaces/canvas/persistence/canvas-persistence-coordinator.ts';text=read(name)
if 'export interface EditorTransactionTarget' not in text:
    text+='''
export interface EditorTransactionTarget {
  <T extends { document: SpaceEditorDocument; revision: number }>(commit: (document: SpaceEditorDocument, revision: number) => Promise<T>): Promise<T>;
}
const transactionTargets = new Map<string, EditorTransactionTarget>();
export function registerCanvasTransactionTarget(spaceId: string, target: EditorTransactionTarget) {
  if (transactionTargets.has(spaceId)) throw new Error(`An editor transaction target already exists for ${spaceId}`);
  transactionTargets.set(spaceId, target);
  return () => { if (transactionTargets.get(spaceId) === target) transactionTargets.delete(spaceId); };
}
export async function transactSpaceEditor<T extends { document: SpaceEditorDocument; revision: number }>(spaceId: string, commit: (document: SpaceEditorDocument, revision: number) => Promise<T>): Promise<T> {
  const active = transactionTargets.get(spaceId); if (active) return active(commit);
  const lease = acquireCanvasPersistenceLease(spaceId); await lease.wait();
  const operation = (async () => { const stored = await canvasStateRepository.load(spaceId); return commit(stored?.document ?? createCanvasDocument([]), stored?.revision ?? 0); })();
  lease.release(operation); return operation;
}
'''
write(name,text)
# Camera remains a separately captured presentation update; stale editor geometry is never echoed into autosave.
name='src/features/spaces/canvas/host/CanvasSurfaceHost.vue';text=read(name)
text=text.replace('import { useCanvasDocumentSession }', 'import { rebaseCanvasDocument } from "./canvas-document-rebase";\nimport { useCanvasDocumentSession }')
text=text.replace('  ...props.events,','  ...props.events,\n  onViewportChange(viewport) { if (session.document.value) session.documentChanged({ ...session.document.value, viewport }); props.events.onViewportChange(viewport); },',1)
text=text.replace('props.events.onCommitGesture({ ...request, baseRevision })', 'props.events.onCommitGesture({ ...request, baseRevision, document: rebaseCanvasDocument(request.baseDocument, session.document.value!, request.document) })')
text=text.replace('onHistoryTransition(nextDocument, transition)', 'onHistoryTransition(nextDocument, transition, baseDocument)')
text=text.replace('...(await transition({ baseRevision, document: nextDocument })),\n      document: nextDocument,','...(await transition({ baseRevision, document: rebaseCanvasDocument(baseDocument, session.document.value!, nextDocument) })),\n      document: rebaseCanvasDocument(baseDocument, session.document.value!, nextDocument),')
write(name,text)
# Ordinary structure edits materialize a draft Space before attempting its first commit.
name='src/features/spaces/canvas/konva/use-konva-canvas-runtime.ts';text=read(name)
text=text.replace('commitDocument: (document, before) => props.events.onCommitGesture({ document, baseDocument: before }),', 'commitDocument: async (document, before) => { await props.events.onPrepareStructureEdit(); return props.events.onCommitGesture({ document, baseDocument: before }); },')
write(name,text)
name='src/features/spaces/presentation/canvas/SpaceCanvas.vue';text=read(name)
text=text.replace('const canvasEvents: CanvasHostEvents = {', 'const canvasEvents: CanvasHostEvents = {\n  onPrepareStructureEdit: store.ensureActiveSpacePersisted,\n  onStructureToolFinished: tools.resetTool,')
text=text.replace('document.nodes !== previous.nodes', 'document.structure !== previous.structure || document.layout !== previous.layout')
write(name,text)
# Prototype key names and projected content scale are handled at the renderer-only boundary.
name='src/features/spaces/canvas/konva/node-content-height-batch.ts';text=read(name)
text=text.replace('layout.frame.width !== measurement.width','Math.abs(layout.frame.width / (layout.contentScale ?? 1) - measurement.width) > 0.01')
write(name,text)
for name,text in files.items():
    path=ROOT/name;path.parent.mkdir(parents=True,exist_ok=True)
    if not path.exists() or path.read_text()!=text:path.write_text(text)
print('Integrated root-aware gestures, structural commands, and subtree deletion routing.')
