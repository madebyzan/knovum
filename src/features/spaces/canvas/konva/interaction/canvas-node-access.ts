import type Konva from "konva";

export function captureCanvasNodeRefs(stage: Konva.Stage | undefined, nodeIds: Iterable<string>) {
  const ids = new Set(nodeIds);
  return new Map(
    (stage?.find((node: Konva.Node) => ids.has(node.getAttr("nodeId"))) ?? []).map(
      (node) => [node.getAttr("nodeId") as string, node] as const,
    ),
  );
}
