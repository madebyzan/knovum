import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { requireMovableScope, resolveCanvasMovementScope } from "../../organization/movement-scope";
import type { CanvasNodeFrameUpdate, CanvasNodeLayoutV1 } from "../contract/canvas-document";

/** Expand translations, never resize operations, through the same operation roots as drag/nudge. */
export function expandCanvasMovement(
  nodes: readonly SpaceCanvasNode[],
  layouts: readonly CanvasNodeLayoutV1[],
  updates: readonly CanvasNodeFrameUpdate[],
): CanvasNodeFrameUpdate[] {
  const scope = requireMovableScope(
    resolveCanvasMovementScope(nodes, new Set(updates.map(({ nodeId }) => nodeId))),
  );
  const byId = new Map(layouts.map((layout) => [layout.nodeId, layout]));
  const requested = new Map(updates.map((update) => [update.nodeId, update]));
  const offsets = new Map(
    [...scope.rootIds].map((id) => {
      const before = byId.get(id);
      if (!before) throw new Error(`The movement root has no layout: ${id}`);
      const after = requested.get(id)!;
      if (!sameExtent(before.frame, after.frame))
        throw new Error("A movement command cannot resize or rotate content");
      return [id, { x: after.frame.x - before.frame.x, y: after.frame.y - before.frame.y }] as const;
    }),
  );
  return [...scope.affectedIds].map((id) => {
    const layout = byId.get(id);
    if (!layout) throw new Error(`The affected canvas item has no layout: ${id}`);
    const delta = offsets.get(scope.ownerById.get(id)!)!;
    return {
      nodeId: id,
      frame: { ...layout.frame, x: layout.frame.x + delta.x, y: layout.frame.y + delta.y },
    };
  });
}

function sameExtent(left: CanvasNodeFrameUpdate["frame"], right: CanvasNodeFrameUpdate["frame"]) {
  return left.width === right.width && left.height === right.height && left.rotation === right.rotation;
}
