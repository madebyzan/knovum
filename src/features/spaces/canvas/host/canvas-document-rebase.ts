import {
  sameCanvasNodeAppearance,
  type CanvasDocumentV1,
  type CanvasNodeFrame,
  type CanvasNodeLayoutV1,
} from "../contract/canvas-document";

export function rebaseCanvasDocument(
  base: CanvasDocumentV1,
  committed: CanvasDocumentV1,
  pending: CanvasDocumentV1,
): CanvasDocumentV1 {
  return {
    version: 1,
    viewport: sameViewport(pending.viewport, base.viewport) ? committed.viewport : pending.viewport,
    nodes: rebaseNodes(base.nodes, committed.nodes, pending.nodes),
    ...withRelationStyles(
      rebaseRelationStyles(base.relationStyles, committed.relationStyles, pending.relationStyles),
    ),
  };
}

function rebaseRelationStyles(
  base: CanvasDocumentV1["relationStyles"],
  committed: CanvasDocumentV1["relationStyles"],
  pending: CanvasDocumentV1["relationStyles"],
) {
  const baseById = new Map(base?.map((style) => [style.relationId, style]));
  const committedById = new Map(committed?.map((style) => [style.relationId, style]));
  const pendingById = new Map(pending?.map((style) => [style.relationId, style]));
  const relationIds = new Set([...committedById.keys(), ...pendingById.keys()]);
  return [...relationIds].flatMap((relationId) => {
    const before = baseById.get(relationId);
    const local = pendingById.get(relationId);
    const remote = committedById.get(relationId);
    const selected = local?.color === before?.color ? remote : local;
    return selected ? [selected] : [];
  });
}

function withRelationStyles(relationStyles: NonNullable<CanvasDocumentV1["relationStyles"]>) {
  return relationStyles.length ? { relationStyles } : {};
}

function rebaseNodes(
  baseNodes: CanvasNodeLayoutV1[],
  committedNodes: CanvasNodeLayoutV1[],
  pendingNodes: CanvasNodeLayoutV1[],
) {
  const baseById = new Map(baseNodes.map((node) => [node.nodeId, node]));
  const pendingById = new Map(pendingNodes.map((node) => [node.nodeId, node]));
  const nodes: CanvasNodeLayoutV1[] = [];

  for (const committedNode of committedNodes) {
    const baseNode = baseById.get(committedNode.nodeId);
    const pendingNode = pendingById.get(committedNode.nodeId);
    if (baseNode && !pendingNode) continue;
    nodes.push(baseNode && pendingNode ? rebaseNode(baseNode, committedNode, pendingNode) : committedNode);
  }

  const committedIds = new Set(committedNodes.map(({ nodeId }) => nodeId));
  for (const pendingNode of pendingNodes) {
    if (!baseById.has(pendingNode.nodeId) && !committedIds.has(pendingNode.nodeId)) nodes.push(pendingNode);
  }
  return nodes;
}

function rebaseNode(
  base: CanvasNodeLayoutV1,
  committed: CanvasNodeLayoutV1,
  pending: CanvasNodeLayoutV1,
): CanvasNodeLayoutV1 {
  return {
    nodeId: committed.nodeId,
    frame: sameFrame(pending.frame, base.frame) ? committed.frame : pending.frame,
    sizeAuthority:
      pending.sizeAuthority === base.sizeAuthority ? committed.sizeAuthority : pending.sizeAuthority,
    appearance: sameCanvasNodeAppearance(pending.appearance, base.appearance)
      ? committed.appearance
      : pending.appearance,
    fileSubpath: pending.fileSubpath === base.fileSubpath ? committed.fileSubpath : pending.fileSubpath,
    zIndex: pending.zIndex === base.zIndex ? committed.zIndex : pending.zIndex,
  };
}

function sameFrame(left: CanvasNodeFrame, right: CanvasNodeFrame) {
  return (
    left.x === right.x &&
    left.y === right.y &&
    left.width === right.width &&
    left.height === right.height &&
    left.rotation === right.rotation
  );
}

function sameViewport(left: CanvasDocumentV1["viewport"], right: CanvasDocumentV1["viewport"]) {
  return left.x === right.x && left.y === right.y && left.zoom === right.zoom;
}
