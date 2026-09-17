import type { SpaceCanvasNode } from "../../../model/nodes/canvas-node";
import type { CanvasNodeLayoutV1 } from "../../contract/canvas-document";
import { spaceNodeDimensions } from "../../node-size";

export function sectionAssignments(
  movedNodeIds: readonly string[],
  layouts: readonly CanvasNodeLayoutV1[],
  sectionNodeIds: ReadonlySet<string>,
  nodes: readonly SpaceCanvasNode[],
) {
  const nodesById = new Map(nodes.map((node) => [node.id, node]));
  const sections = layouts
    .filter(({ nodeId }) => sectionNodeIds.has(nodeId))
    .map((layout) =>
      Object.assign({}, layout, { frame: visibleSectionFrame(layout, nodesById.get(layout.nodeId)) }),
    );
  const byId = new Map(layouts.map((layout) => [layout.nodeId, layout]));
  const currentSections = new Map(nodes.map(({ id, data }) => [id, data.sectionId]));
  return movedNodeIds.flatMap<{ id: string; sectionId?: string }>((id) => {
    if (sectionNodeIds.has(id)) return [];
    const node = byId.get(id);
    if (!node) return [];
    const center = {
      x: node.frame.x + node.frame.width / 2,
      y: node.frame.y + node.frame.height / 2,
    };
    const parent = sections
      .filter(({ nodeId }) => nodeId !== id)
      .filter(({ frame }) => contains(frame, center))
      .toSorted((left, right) => area(left.frame) - area(right.frame))[0];
    if (parent?.nodeId === currentSections.get(id)) return [];
    return [{ id, ...(parent ? { sectionId: parent.nodeId } : {}) }];
  });
}

function contains(frame: CanvasNodeLayoutV1["frame"], point: { x: number; y: number }) {
  return (
    point.x >= frame.x &&
    point.x <= frame.x + frame.width &&
    point.y >= frame.y &&
    point.y <= frame.y + frame.height
  );
}

const area = (frame: CanvasNodeLayoutV1["frame"]) => frame.width * frame.height;

function visibleSectionFrame(layout: CanvasNodeLayoutV1, node: SpaceCanvasNode | undefined) {
  if (node?.data.kind !== "section" || !node.data.collapsed) return layout.frame;
  return { ...layout.frame, ...spaceNodeDimensions(node.data) };
}
