import type Konva from "konva";

import type { CanvasNodeFrameUpdate, CanvasNodeLayoutV1 } from "../../schema/document";
import { captureCanvasNodeRefs } from "./canvas-node-access";
import { createCanvasTranslationSnapIndex } from "./canvas-translation-snap-index";
import type { CanvasTranslationGestureState } from "./canvas-translation-snapping";

interface CanvasNodeDragOptions {
  layouts: readonly CanvasNodeLayoutV1[];
  selectedNodeIds: ReadonlySet<string>;
  lockedNodeIds: ReadonlySet<string>;
  primaryId: string;
  stage: Konva.Stage | undefined;
}

export function captureCanvasNodeDragState(options: CanvasNodeDragOptions) {
  const { layouts, selectedNodeIds, lockedNodeIds, primaryId, stage } = options;
  if (lockedNodeIds.has(primaryId)) return undefined;
  const movable = movableCanvasSelectionNodeIds(new Set(selectedNodeIds).add(primaryId), lockedNodeIds);
  const layoutByNodeId = new Map(layouts.map((layout) => [layout.nodeId, layout]));
  const frameByNodeId = new Map(
    [...movable].flatMap((id) => {
      const frame = layoutByNodeId.get(id)?.frame;
      return frame ? [[id, { ...frame }] as const] : [];
    }),
  );
  const origin = frameByNodeId.get(primaryId);
  if (!origin) return undefined;
  const stationary = layouts.filter(({ nodeId }) => !frameByNodeId.has(nodeId)).map(({ frame }) => frame);
  const state: CanvasTranslationGestureState = {
    primaryId,
    origin,
    frameByNodeId,
    snapIndex: createCanvasTranslationSnapIndex(stationary),
  };
  const nodeRefs = captureCanvasNodeRefs(stage, frameByNodeId.keys());
  return { nodeRefs, state };
}

export function movableCanvasSelectionNodeIds(
  selectedNodeIds: ReadonlySet<string>,
  lockedNodeIds: ReadonlySet<string>,
) {
  return new Set([...selectedNodeIds].filter((id) => !lockedNodeIds.has(id)));
}

export function restoreCanvasNodeFrames(
  updates: readonly CanvasNodeFrameUpdate[],
  nodeById: ReadonlyMap<string, Konva.Node>,
) {
  for (const { nodeId, frame } of updates) {
    const node = nodeById.get(nodeId);
    node?.position({ x: frame.x, y: frame.y });
    node?.rotation(frame.rotation);
    node?.scale({ x: 1, y: 1 });
  }
}
