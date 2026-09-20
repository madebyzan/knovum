import type Konva from "konva";

import type { CanvasDisplayNode } from "../../contract/canvas-display-node";
import type {
  CanvasNodeFrameUpdate,
  CanvasNodeLayoutV1,
  SpaceEditorDocument,
} from "../../contract/canvas-document";
import { expandAggregatePreview } from "./aggregate-preview";
import type { CanvasGestureUpdate } from "./canvas-gesture-update";
import { captureCanvasNodeRefs } from "./canvas-node-access";
import { captureCanvasNodeDragState, restoreCanvasNodeFrames } from "./canvas-node-drag-state";
import type { CanvasGestureFeedback, CanvasSnapGuide } from "./canvas-snapping";
import { captureCanvasTransformTargets, projectCanvasTransformUpdates } from "./canvas-transform-projection";
import { snapCanvasTranslation, type CanvasTranslationGestureState } from "./canvas-translation-snapping";
import { projectResizeFeedback } from "./proportional-resize-feedback";
interface NodeGestureControllerOptions {
  document(): SpaceEditorDocument;
  layouts(): readonly CanvasNodeLayoutV1[];
  nodes(): readonly CanvasDisplayNode[];
  previewFrames(updates: readonly CanvasGestureUpdate[]): void;
  selectedNodeIds(): ReadonlySet<string>;
  stage(): Konva.Stage | undefined;
  updateFeedback(feedback: CanvasGestureFeedback | undefined): void;
  zoom(): number;
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
    this.resizeGuides = [];
    this.options.updateFeedback(undefined);
    const captured = captureCanvasNodeDragState(
      this.options.layouts(),
      this.options.document(),
      this.options.selectedNodeIds(),
      primaryId,
      this.options.stage(),
    );
    if (!captured) {
      captureCanvasNodeRefs(this.options.stage(), [primaryId]).get(primaryId)?.stopDrag();
      return false;
    }
    this.drag = captured.state;
    this.dragNodes = captured.nodeRefs;
    return true;
  };
  readonly moveDrag = (primaryId: string) => {
    if (!this.drag || this.drag.primaryId !== primaryId) return;
    const primary = this.dragNodes.get(primaryId);
    if (!primary) return;
    const proposed = { x: primary.x() - this.drag.origin.x, y: primary.y() - this.drag.origin.y };
    const snapped = snapCanvasTranslation(
      [...this.drag.frames.values()],
      this.drag.snapIndex,
      proposed,
      6 / this.options.zoom(),
    );
    const updates = [...this.drag.frames].map(([nodeId, frame]) => ({
      nodeId,
      frame: { ...frame, x: frame.x + snapped.x, y: frame.y + snapped.y },
    }));
    for (const update of updates) this.dragNodes.get(update.nodeId)?.position(update.frame);
    this.options.previewFrames(updates);
    this.options.updateFeedback({ guides: snapped.guides });
  };
  endDrag(primaryId: string): CanvasGestureUpdate[] {
    if (!this.drag || this.drag.primaryId !== primaryId) return [];
    const current = this.drag;
    const primary = this.dragNodes.get(primaryId);
    this.drag = undefined;
    this.dragNodes.clear();
    this.options.updateFeedback(undefined);
    if (!primary) return [];
    const delta = { x: primary.x() - current.origin.x, y: primary.y() - current.origin.y };
    return [...current.frames].map(([nodeId, frame]) => ({
      nodeId,
      frame: { ...frame, x: frame.x + delta.x, y: frame.y + delta.y },
    }));
  }
  beginTransform() {
    this.transformLayouts = this.options.layouts();
    this.transformTargets = captureCanvasTransformTargets(
      this.transformLayouts,
      this.options.nodes(),
      this.options.selectedNodeIds(),
      this.options.stage(),
    );
    this.transforming = true;
  }
  readonly previewTransform = (anchor?: string) => {
    this.proportionalAnchor = anchor;
    const updates = this.transformedLayouts();
    const expanded = expandAggregatePreview(this.options.document(), this.transformLayouts, updates);
    const roots = new Set(updates.map((update) => update.nodeId));
    const refs = captureCanvasNodeRefs(
      this.options.stage(),
      expanded.map((update) => update.nodeId),
    );
    const before = new Map(this.transformLayouts.map((layout) => [layout.nodeId, layout.frame]));
    for (const update of expanded) {
      if (roots.has(update.nodeId)) continue;
      const node = refs.get(update.nodeId),
        frame = before.get(update.nodeId)!;
      node?.position(update.frame);
      node?.rotation(update.frame.rotation);
      node?.scale({ x: update.frame.width / frame.width, y: update.frame.height / frame.height });
    }
    this.options.previewFrames(expanded);
    const frame = updates[0]?.frame;
    this.options.updateFeedback(
      frame ? projectResizeFeedback(frame, this.resizeGuides, this.options.zoom(), anchor) : undefined,
    );
  };
  endTransform() {
    const updates = this.transformedLayouts();
    this.transforming = false;
    this.proportionalAnchor = undefined;
    this.transformTargets = [];
    this.options.updateFeedback(undefined);
    return updates;
  }
  panDrag(delta: { x: number; y: number }) {
    if (!this.drag) return;
    const node = this.dragNodes.get(this.drag.primaryId);
    if (!node) return;
    node.position({ x: node.x() + delta.x, y: node.y() + delta.y });
    this.moveDrag(this.drag.primaryId);
  }
  readonly updateResizeGuides = (guides: readonly CanvasSnapGuide[]) => {
    this.resizeGuides = guides;
  };
  cancelActiveGesture() {
    if (!this.drag && !this.transforming) return false;
    const frames = this.drag
      ? [...this.drag.frames].map(([nodeId, frame]) => ({ nodeId, frame }))
      : this.transformLayouts;
    this.restoreFrames(frames);
    this.drag = undefined;
    this.dragNodes.clear();
    this.transforming = false;
    this.transformTargets = [];
    this.proportionalAnchor = undefined;
    this.resizeGuides = [];
    this.options.previewFrames([]);
    this.options.updateFeedback(undefined);
    return true;
  }
  restoreFrames(
    updates: readonly CanvasNodeFrameUpdate[],
    nodes = captureCanvasNodeRefs(
      this.options.stage(),
      updates.map((update) => update.nodeId),
    ),
  ) {
    restoreCanvasNodeFrames(updates, nodes);
  }
  private transformedLayouts() {
    return projectCanvasTransformUpdates({
      proportionalAnchor: this.proportionalAnchor,
      targets: this.transformTargets,
    });
  }
}
