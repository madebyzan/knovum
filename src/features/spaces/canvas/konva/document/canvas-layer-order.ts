import type { CanvasDocumentV1 } from "../../contract/canvas-document";
import type { CanvasLayerMove } from "../../contract/canvas-editor";

type CanvasLayout = CanvasDocumentV1["nodes"][number];

export function reorderCanvasNodes(
  nodes: readonly CanvasLayout[],
  selected: ReadonlySet<string>,
  mode: CanvasLayerMove,
) {
  const ordered = nodes.toSorted((left, right) => left.zIndex - right.zIndex);
  if (!ordered.some(({ nodeId }) => selected.has(nodeId))) return [...nodes];
  if (mode === "front" || mode === "back") {
    const chosen = ordered.filter(({ nodeId }) => selected.has(nodeId));
    const rest = ordered.filter(({ nodeId }) => !selected.has(nodeId));
    return normalize(mode === "front" ? [...rest, ...chosen] : [...chosen, ...rest]);
  }
  const moved = [...ordered];
  if (mode === "forward") stepForward(moved, selected);
  else stepBackward(moved, selected);
  return normalize(moved);
}

function normalize(nodes: CanvasLayout[]) {
  return nodes.map((node, zIndex) => (node.zIndex === zIndex ? node : { ...node, zIndex }));
}

function stepForward(nodes: CanvasLayout[], selected: ReadonlySet<string>) {
  for (let index = nodes.length - 2; index >= 0; index -= 1) {
    const current = nodes[index];
    const next = nodes[index + 1];
    if (!current || !next || !selected.has(current.nodeId) || selected.has(next.nodeId)) continue;
    [nodes[index], nodes[index + 1]] = [next, current];
  }
}

function stepBackward(nodes: CanvasLayout[], selected: ReadonlySet<string>) {
  for (let index = 1; index < nodes.length; index += 1) {
    const current = nodes[index];
    const previous = nodes[index - 1];
    if (!current || !previous || !selected.has(current.nodeId) || selected.has(previous.nodeId)) continue;
    [nodes[index - 1], nodes[index]] = [current, previous];
  }
}
