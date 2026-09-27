import { isEqual } from "es-toolkit";

import type { SpaceEditSnapshot } from "../../model/structure/edit-snapshot";
import type { LocalTransform } from "../../model/structure/structure-contract";
import { createStructureIndex, effectiveOperationRoots } from "../../model/structure/structure-index";
import { worldTransformFor } from "../../model/structure/world-transform";
import type { SpaceEditCommand } from "../../structure/space-edit-command";
import { serializeCanvasAppearance } from "../contract/canvas-document";
import type { CanvasDocumentV1, CanvasNodeLayoutV1 } from "../contract/canvas-document";

const EPSILON = 1e-6;

/** Translate the existing Konva gesture adapter output into one authoritative product command. */
export function commandForCanvasGesture(
  snapshot: SpaceEditSnapshot,
  before: CanvasDocumentV1,
  after: CanvasDocumentV1,
): SpaceEditCommand | undefined {
  const previousByNode = new Map(before.nodes.map((layout) => [layout.nodeId, layout]));
  const changed = after.nodes.filter((layout) => {
    const previous = previousByNode.get(layout.nodeId);
    return previous && !sameLayout(previous, layout);
  });
  if (!changed.length) return undefined;

  const index = createStructureIndex(snapshot.structure);
  const structureNodeIds = changed.map(({ nodeId }) => {
    const structureNodeId = index.nodeIdToStructureNodeId.get(nodeId);
    if (!structureNodeId) throw new Error(`Content structure not found: ${nodeId}`);
    return structureNodeId;
  });
  const roots = effectiveOperationRoots(structureNodeIds, index).map((structureNode) => structureNode.id);
  const first = changed[0]!;
  const firstBefore = previousByNode.get(first.nodeId)!;
  const delta = { x: first.frame.x - firstBefore.frame.x, y: first.frame.y - firstBefore.frame.y };
  if (
    changed.every((layout) => {
      const previous = previousByNode.get(layout.nodeId)!;
      return (
        nearly(layout.frame.x - previous.frame.x, delta.x) &&
        nearly(layout.frame.y - previous.frame.y, delta.y) &&
        sameFrameShape(previous, layout) &&
        sameAppearance(previous, layout)
      );
    })
  ) {
    return nearly(delta.x, 0) && nearly(delta.y, 0)
      ? undefined
      : { kind: "structure.move", rootIds: roots, delta };
  }

  if (changed.length !== 1) throw new Error("A multi-node transform must be uniform");
  return contentLayoutCommand(snapshot, structureNodeIds[0]!, first);
}

function contentLayoutCommand(
  snapshot: SpaceEditSnapshot,
  structureNodeId: string,
  next: CanvasNodeLayoutV1,
): SpaceEditCommand {
  const index = createStructureIndex(snapshot.structure);
  const layouts = new Map(snapshot.layouts.map((layout) => [layout.structureNodeId, layout]));
  const structureNode = index.structureById.get(structureNodeId);
  const current = layouts.get(structureNodeId);
  if (structureNode?.kind !== "content" || current?.kind !== "content")
    throw new Error(`Content layout not found: ${structureNodeId}`);
  const parentWorld: LocalTransform =
    structureNode.parentId === null
      ? { x: 0, y: 0, rotation: 0, scale: 1 }
      : worldTransformFor(structureNode.parentId, index, layouts);
  const transform = localFromWorld(
    {
      x: next.frame.x,
      y: next.frame.y,
      rotation: next.frame.rotation,
      scale: worldTransformFor(structureNodeId, index, layouts).scale,
    },
    parentWorld,
  );
  return {
    kind: "content.update-layout",
    structureNodeId,
    transform,
    size: {
      width: next.frame.width / (parentWorld.scale * transform.scale),
      height: next.frame.height / (parentWorld.scale * transform.scale),
    },
    sizeAuthority: next.sizeAuthority,
    appearance: next.appearance
      ? { kind: "set", value: serializeCanvasAppearance(next.appearance) }
      : { kind: "clear" },
  };
}

function localFromWorld(world: LocalTransform, parent: LocalTransform): LocalTransform {
  const radians = (parent.rotation * Math.PI) / 180;
  const dx = world.x - parent.x;
  const dy = world.y - parent.y;
  return {
    x: (dx * Math.cos(radians) + dy * Math.sin(radians)) / parent.scale,
    y: (-dx * Math.sin(radians) + dy * Math.cos(radians)) / parent.scale,
    rotation: world.rotation - parent.rotation,
    scale: world.scale / parent.scale,
  };
}

function sameLayout(left: CanvasNodeLayoutV1, right: CanvasNodeLayoutV1) {
  return (
    nearly(left.frame.x, right.frame.x) &&
    nearly(left.frame.y, right.frame.y) &&
    sameFrameShape(left, right) &&
    left.sizeAuthority === right.sizeAuthority &&
    sameAppearance(left, right)
  );
}

function sameFrameShape(left: CanvasNodeLayoutV1, right: CanvasNodeLayoutV1) {
  return (
    nearly(left.frame.width, right.frame.width) &&
    nearly(left.frame.height, right.frame.height) &&
    nearly(left.frame.rotation, right.frame.rotation) &&
    left.sizeAuthority === right.sizeAuthority
  );
}

function sameAppearance(left: CanvasNodeLayoutV1, right: CanvasNodeLayoutV1) {
  return isEqual(left.appearance, right.appearance);
}

function nearly(left: number, right: number) {
  return Math.abs(left - right) <= EPSILON;
}
