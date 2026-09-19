import type Konva from "konva";

import type { SpaceCanvasNode } from "../../../model/nodes/canvas-node";
import { resolveCanvasMovementScope } from "../../../organization/movement-scope";
import type { CanvasNodeFrameUpdate, CanvasNodeLayoutV1 } from "../../contract/canvas-document";
import { captureCanvasNodeRefs } from "./canvas-node-access";
import {
  createCanvasTranslationSnapIndex,
  type CanvasTranslationGestureState,
} from "./canvas-translation-snapping";

interface CanvasNodeDragInput {
  layouts: readonly CanvasNodeLayoutV1[];
  nodes: readonly SpaceCanvasNode[];
  selectedNodeIds: ReadonlySet<string>;
  primaryId: string;
  stage: Konva.Stage | undefined;
}

export function captureCanvasNodeDragState({
  layouts,
  nodes,
  selectedNodeIds,
  primaryId,
  stage,
}: CanvasNodeDragInput) {
  const scope = resolveCanvasMovementScope(nodes, new Set([...selectedNodeIds, primaryId]));
  if (scope.blockedIds.length) return undefined;
  const byId = new Map(layouts.map((layout) => [layout.nodeId, layout]));
  const frames = new Map(
    [...scope.affectedIds].map((id) => {
      const frame = byId.get(id)?.frame;
      if (!frame) throw new Error(`The affected canvas item has no layout: ${id}`);
      return [id, { ...frame }] as const;
    }),
  );
  const origin = frames.get(primaryId);
  if (!origin) return undefined;
  const stationary = layouts.filter(({ nodeId }) => !frames.has(nodeId)).map(({ frame }) => frame);
  const state: CanvasTranslationGestureState = {
    primaryId,
    origin,
    frames,
    snapIndex: createCanvasTranslationSnapIndex(stationary),
  };
  const nodeRefs = captureCanvasNodeRefs(stage, frames.keys());
  return { nodeRefs, state };
}

export function restoreCanvasNodeFrames(
  updates: readonly CanvasNodeFrameUpdate[],
  nodes: ReadonlyMap<string, Konva.Node>,
) {
  for (const { nodeId, frame } of updates) {
    const node = nodes.get(nodeId);
    node?.position({ x: frame.x, y: frame.y });
    node?.rotation(frame.rotation);
    node?.scale({ x: 1, y: 1 });
  }
}
