import type Konva from "konva";

import type { CanvasNodeFrameUpdate } from "../../schema/document";

export function captureCanvasNodeRefs(stage: Konva.Stage | undefined, nodeIds: Iterable<string>) {
  const ids = new Set(nodeIds);
  return new Map(
    (stage?.find((node: Konva.Node) => ids.has(node.getAttr("nodeId"))) ?? []).map(
      (node) => [node.getAttr("nodeId") as string, node] as const,
    ),
  );
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
