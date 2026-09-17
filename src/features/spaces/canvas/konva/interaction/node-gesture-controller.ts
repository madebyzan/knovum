import type Konva from "konva";

import type { SpaceCanvasNode } from "../../../model/nodes/canvas-node";
import type { CanvasNodeFrameUpdate, CanvasNodeLayoutV1 } from "../../contract/canvas-document";
import type { CanvasGestureUpdate } from "./canvas-gesture-update";
import { captureCanvasNodeRefs } from "./canvas-node-access";
import { captureCanvasNodeDragState, restoreCanvasNodeFrames } from "./canvas-node-drag-state";
import type { CanvasGestureFeedback, CanvasSnapGuide } from "./canvas-snapping";
import { captureCanvasTransformTargets, projectCanvasTransformUpdates } from "./canvas-transform-projection";
import { snapCanvasTranslation, type CanvasTranslationGestureState } from "./canvas-translation-snapping";
import { projectResizeFeedback } from "./proportional-resize-feedback";

interface NodeGestureControllerOptions {
  layouts(): readonly CanvasNodeLayoutV1[];
  nodes(): readonly SpaceCanvasNode[];
  previewFrames(updates: readonly CanvasGestureUpdate[]): void;
  selectedNodeIds(): ReadonlySet<string>;
  stage(): Konva.Stage | undefined;
  updateFeedback(feedback: CanvasGestureFeedback | undefined): void;
  zoom(): number;
}

export class NodeGestureController {
  private drag: CanvasTranslationGestureState | undefined;

  constructor(private readonly options: NodeGestureControllerOptions) {}

  readonly beginDrag = (primaryId: string) => {
    this.resizeGuides = [];
    this.options.updateFeedback(undefined);
    const captured = captureCanvasNodeDragState(
      this.options.layouts(),
      this.options.nodes(),
      this.options.selectedNodeIds(),
      primaryId,
      this.options.stage(),
    );
    if (!captured) return;
    this.drag = captured.state;
    this.dragNodes = captured.nodeRefs;
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
    const offset = { x: snapped.x, y: snapped.y };
    primary.position({ x: this.drag.origin.x + offset.x, y: this.drag.origin.y + offset.y });
    const updates = [...this.drag.frames].map(([nodeId, frame]) => ({
      nodeId,
      frame: { ...frame, x: frame.x + offset.x, y: frame.y + offset.y },
    }));
    for (const [nodeId, frame] of this.drag.frames) {
      if (nodeId === primaryId) continue;
      this.dragNodes.get(nodeId)?.position({ x: frame.x + offset.x, y: frame.y + offset.y });
    }
    this.options.previewFrames(updates);
    this.options.updateFeedback({ guides: snapped.guides });
  };

  async endDrag(primaryId: string) {
    if (!this.drag || this.drag.primaryId !== primaryId) return [];
    const current = this.drag;
    this.drag = undefined;
    this.options.updateFeedback(undefined);
    const primary = this.dragNodes.get(primaryId);
    this.dragNodes.clear();
    if (!primary) return [];
    const offset = { x: primary.x() - current.origin.x, y: primary.y() - current.origin.y };
    const updates = [...current.frames].map(([nodeId, frame]) => ({
      nodeId,
      frame: { ...frame, x: frame.x + offset.x, y: frame.y + offset.y },
    }));
    return updates;
  }

  async endTransform() {
    const updates = this.transformedLayouts();
    for (const { node } of this.transformTargets) node.scale({ x: 1, y: 1 });
    this.transforming = false;
    this.proportionalAnchor = undefined;
    this.transformTargets = [];
    this.options.previewFrames(updates);
    this.options.updateFeedback(undefined);
    return updates;
  }

  readonly previewTransform = (proportionalAnchor?: string) => {
    this.transforming = true;
    this.proportionalAnchor = proportionalAnchor;
    const updates = this.transformedLayouts();
    this.options.previewFrames(updates);
    const frame = updates[0]?.frame;
    this.options.updateFeedback(
      frame
        ? projectResizeFeedback(frame, this.resizeGuides, this.options.zoom(), proportionalAnchor)
        : undefined,
    );
  };

  beginTransform() {
    this.transformTargets = captureCanvasTransformTargets(
      this.options.layouts(),
      this.options.nodes(),
      this.options.selectedNodeIds(),
      this.options.stage(),
    );
  }

  panDrag(offset: { x: number; y: number }) {
    if (!this.drag) return;
    const primary = this.dragNodes.get(this.drag.primaryId);
    if (!primary) return;
    primary.position({ x: primary.x() + offset.x, y: primary.y() + offset.y });
    this.moveDrag(this.drag.primaryId);
  }

  readonly updateResizeGuides = (guides: readonly CanvasSnapGuide[]) => {
    this.resizeGuides = guides;
  };

  cancelActiveGesture() {
    if (!this.drag && !this.transforming) return false;
    const frames = this.drag
      ? [...this.drag.frames].map(([nodeId, frame]) => ({ nodeId, frame }))
      : this.options
          .layouts()
          .filter(({ nodeId }) => this.options.selectedNodeIds().has(nodeId))
          .map(({ nodeId, frame }) => ({ nodeId, frame }));
    const gestureNodes = new Map([
      ...this.dragNodes,
      ...this.transformTargets.map(({ content, node }) => [content.id, node] as const),
    ]);
    this.drag = undefined;
    this.transforming = false;
    this.proportionalAnchor = undefined;
    this.resizeGuides = [];
    this.restoreFrames(frames, gestureNodes);
    this.dragNodes.clear();
    this.transformTargets = [];
    this.options.previewFrames([]);
    this.options.updateFeedback(undefined);
    return true;
  }

  restoreFrames(
    updates: readonly CanvasNodeFrameUpdate[],
    nodes = captureCanvasNodeRefs(
      this.options.stage(),
      updates.map(({ nodeId }) => nodeId),
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

  private dragNodes = new Map<string, Konva.Node>();
  private resizeGuides: readonly CanvasSnapGuide[] = [];
  private transforming = false;
  private proportionalAnchor: string | undefined;
  private transformTargets: ReturnType<typeof captureCanvasTransformTargets> = [];
}
