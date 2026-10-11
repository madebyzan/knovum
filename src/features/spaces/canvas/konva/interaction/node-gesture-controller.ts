import type Konva from "konva";

import type { SpaceCanvasNode } from "../../../model/nodes/canvas-node";
import type { CanvasNodeFrameUpdate, CanvasNodeLayoutV1 } from "../../schema/document";
import type { CanvasGestureUpdate } from "./canvas-gesture-update";
import { captureCanvasNodeRefs, restoreCanvasNodeFrames } from "./canvas-node-access";
import type { CanvasGestureFeedback, CanvasSnapGuide } from "./canvas-snapping";
import { captureCanvasTransformTargets, projectCanvasTransformUpdates } from "./canvas-transform-projection";
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
  constructor(private readonly options: NodeGestureControllerOptions) {}

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
        ? projectResizeFeedback({
            frame,
            snapGuides: this.resizeGuides,
            zoom: this.options.zoom(),
            proportionalAnchor,
          })
        : undefined,
    );
  };

  beginTransform() {
    this.transformTargets = captureCanvasTransformTargets({
      layouts: this.options.layouts(),
      nodes: this.options.nodes(),
      selectedNodeIds: this.options.selectedNodeIds(),
      stage: this.options.stage(),
    });
  }

  readonly updateResizeGuides = (guides: readonly CanvasSnapGuide[]) => {
    this.resizeGuides = guides;
  };

  cancelActiveGesture() {
    if (!this.transforming) return false;
    const frameUpdates = this.options
      .layouts()
      .filter(({ nodeId }) => this.options.selectedNodeIds().has(nodeId))
      .map(({ nodeId, frame }) => ({ nodeId, frame }));
    const gestureNodeById = new Map(
      this.transformTargets.map(({ content, node }) => [content.id, node] as const),
    );
    this.transforming = false;
    this.proportionalAnchor = undefined;
    this.resizeGuides = [];
    this.restoreFrames(frameUpdates, gestureNodeById);
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

  private resizeGuides: readonly CanvasSnapGuide[] = [];
  private transforming = false;
  private proportionalAnchor: string | undefined;
  private transformTargets: ReturnType<typeof captureCanvasTransformTargets> = [];
}
