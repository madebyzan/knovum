import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { CanvasDocumentV1 } from "../contract/canvas-document";
import { spaceNodeSizePolicy, spaceNodeSupportsContentSizing } from "../node-size";

type CanvasDocumentNode = CanvasDocumentV1["nodes"][number];

export function createCanvasDocument(nodes: readonly SpaceCanvasNode[]): CanvasDocumentV1 {
  return {
    version: 1,
    viewport: { x: 0, y: 0, zoom: 1 },
    nodes: nodes.map(createDocumentNode),
  };
}

export function reconcileCanvasDocument(
  document: CanvasDocumentV1,
  nodes: readonly SpaceCanvasNode[],
): CanvasDocumentV1 {
  const current = new Map(document.nodes.map((node) => [node.nodeId, node]));
  const nextNodes = nodes.map((node, index) => synchronizeDocumentNode(node, current.get(node.id), index));
  if (sameNodeSequence(document.nodes, nextNodes)) return document;
  return { ...document, nodes: nextNodes };
}

function createDocumentNode(node: SpaceCanvasNode, zIndex: number): CanvasDocumentNode {
  const { width, height } = spaceNodeSizePolicy(node.data);
  return {
    nodeId: node.id,
    frame: { ...node.initialPosition, width, height, rotation: 0 },
    sizeAuthority: spaceNodeSupportsContentSizing(node.data) ? "content" : "frame",
    zIndex,
  };
}

function synchronizeDocumentNode(
  node: SpaceCanvasNode,
  current: CanvasDocumentNode | undefined,
  zIndex: number,
): CanvasDocumentNode {
  if (!current) return createDocumentNode(node, zIndex);
  if (current.sizeAuthority === "content" && !spaceNodeSupportsContentSizing(node.data)) {
    throw new Error(`Node ${node.id} does not support content-authoritative sizing`);
  }
  if (current.sizeAuthority === "content") {
    const { width, height } = spaceNodeSizePolicy(node.data);
    if (width !== current.frame.width || height !== current.frame.height) {
      return { ...current, frame: { ...current.frame, width, height } };
    }
  }
  return current;
}

function sameNodeSequence(left: readonly unknown[], right: readonly unknown[]) {
  return left.length === right.length && left.every((value, index) => value === right[index]);
}
