import type Konva from "konva";

import type { SpaceCanvasNode } from "../../../model/nodes/canvas-node";
import type { CanvasNodeFrameUpdate, CanvasNodeLayoutV1 } from "../../contract/canvas-document";
import { captureCanvasNodeRefs } from "./canvas-node-access";
import {
  createCanvasTranslationSnapIndex,
  type CanvasTranslationGestureState,
} from "./canvas-translation-snapping";

interface CaptureCanvasNodeDragStateOptions {
  layouts: readonly CanvasNodeLayoutV1[];
  nodes: readonly SpaceCanvasNode[];
  selectedNodeIds: ReadonlySet<string>;
  primaryId: string;
  stage: Konva.Stage | undefined;
}

export function captureCanvasNodeDragState(options: CaptureCanvasNodeDragStateOptions) {
  const { layouts, nodes, selectedNodeIds, primaryId, stage } = options;
  const locked = new Set(nodes.filter(({ data }) => data.locked).map(({ id }) => id));
  if (locked.has(primaryId)) return undefined;
  const selected = new Set(selectedNodeIds).add(primaryId);
  for (const node of nodes) if (node.data.sectionId === primaryId) selected.add(node.id);
  const byId = new Map(layouts.map((layout) => [layout.nodeId, layout]));
  const frames = new Map(
    [...selected].flatMap((id) => {
      const frame = locked.has(id) ? undefined : byId.get(id)?.frame;
      return frame ? [[id, { ...frame }] as const] : [];
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
