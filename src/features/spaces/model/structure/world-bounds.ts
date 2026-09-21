import {
  MAX_WORLD_COORDINATE,
  type StructureNodeId,
  type StructureLayout,
  type LocalTransform,
} from "./structure-contract";
import type { StructureIndex } from "./structure-index";
import { worldTransformFor } from "./world-transform";

export interface WorldBounds {
  left: number;
  top: number;
  right: number;
  bottom: number;
}

/** Bounds use actual transformed content corners, without recursively rotating AABBs. */
export function boundsForOperationRoots(
  rootIds: readonly StructureNodeId[],
  index: StructureIndex,
  layoutById: ReadonlyMap<StructureNodeId, StructureLayout>,
): WorldBounds | undefined {
  const visited = new Set<StructureNodeId>();
  const pending = [...rootIds];
  let bounds: WorldBounds | undefined;
  while (pending.length) {
    const id = pending.pop()!;
    if (visited.has(id)) continue;
    visited.add(id);
    const structureNode = index.structureById.get(id);
    if (!structureNode) throw new Error(`Structure structureNode not found: ${id}`);
    if (structureNode.kind !== "content") {
      for (const child of index.childrenByParent.get(id) ?? []) pending.push(child.id);
      continue;
    }
    const layout = layoutById.get(id);
    if (layout?.kind !== "content") throw new Error(`Content layout missing: ${id}`);
    bounds = mergeWorldBounds(bounds, contentWorldBounds(layout, worldTransformFor(id, index, layoutById)));
  }
  return bounds;
}

export function contentWorldBounds(
  layout: Extract<StructureLayout, { kind: "content" }>,
  world: LocalTransform,
): WorldBounds {
  const radians = (world.rotation * Math.PI) / 180;
  const cos = Math.cos(radians);
  const sin = Math.sin(radians);
  const corners: readonly (readonly [number, number])[] = [
    [0, 0],
    [layout.size.width, 0],
    [0, layout.size.height],
    [layout.size.width, layout.size.height],
  ];
  let bounds: WorldBounds | undefined;
  for (const [x, y] of corners) {
    const px = world.x + world.scale * (x * cos - y * sin);
    const py = world.y + world.scale * (x * sin + y * cos);
    if (
      !Number.isFinite(px) ||
      !Number.isFinite(py) ||
      Math.abs(px) > MAX_WORLD_COORDINATE ||
      Math.abs(py) > MAX_WORLD_COORDINATE
    )
      throw new Error(`Content geometry exceeds bounds: ${layout.structureNodeId}`);
    bounds = mergeWorldBounds(bounds, { left: px, top: py, right: px, bottom: py });
  }
  if (!bounds) throw new Error(`Content has no geometry: ${layout.structureNodeId}`);
  return bounds;
}

export function mergeWorldBounds(
  left: WorldBounds | undefined,
  right: WorldBounds | undefined,
): WorldBounds | undefined {
  if (!left) return right;
  if (!right) return left;
  return {
    left: Math.min(left.left, right.left),
    top: Math.min(left.top, right.top),
    right: Math.max(left.right, right.right),
    bottom: Math.max(left.bottom, right.bottom),
  };
}
