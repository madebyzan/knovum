import type { StructureNodeId, StructureLayout, LocalTransform } from "./schema/structure";
import { MAX_WORLD_COORDINATE, MAX_WORLD_SCALE } from "./schema/structure";
import { ancestorsOf, type StructureIndex } from "./structure-index";

const IDENTITY: LocalTransform = { x: 0, y: 0, rotation: 0, scale: 1 };

export function localTransformOf(layout: StructureLayout): LocalTransform {
  return layout.kind === "section" ? { ...layout.position, rotation: 0, scale: 1 } : layout.transform;
}

/** Rotation is measured in degrees, matching canvas layout and Konva. */
export function composeTransforms(parent: LocalTransform, local: LocalTransform): LocalTransform {
  const radians = (parent.rotation * Math.PI) / 180;
  const cos = Math.cos(radians);
  const sin = Math.sin(radians);
  const worldTransform = {
    x: parent.x + parent.scale * (local.x * cos - local.y * sin),
    y: parent.y + parent.scale * (local.x * sin + local.y * cos),
    rotation: parent.rotation + local.rotation,
    scale: parent.scale * local.scale,
  };
  assertTransform(worldTransform);
  return worldTransform;
}

export function worldTransformFor(
  id: StructureNodeId,
  index: StructureIndex,
  layoutById: ReadonlyMap<StructureNodeId, StructureLayout>,
): LocalTransform {
  const structureNode = index.structureById.get(id);
  if (!structureNode) throw new Error(`Structure structureNode not found: ${id}`);
  let world = IDENTITY;
  for (const member of [...ancestorsOf(id, index), structureNode]) {
    const layout = layoutById.get(member.id);
    if (!layout || layout.kind !== member.kind)
      throw new Error(`Structure layout missing or mismatched: ${member.id}`);
    world = composeTransforms(world, localTransformOf(layout));
  }
  return world;
}

/** Returns the structureNode's new local layout; caller commits its parent and sibling order. */
export function layoutKeepingWorldPosition(
  id: StructureNodeId,
  newParentId: StructureNodeId | null,
  index: StructureIndex,
  layoutById: ReadonlyMap<StructureNodeId, StructureLayout>,
): StructureLayout {
  const structureNode = index.structureById.get(id);
  const layout = layoutById.get(id);
  if (!structureNode || !layout || structureNode.kind !== layout.kind)
    throw new Error(`Structure structureNode not found: ${id}`);
  if (structureNode.parentId === newParentId) return layout;
  validateReparenting(id, structureNode.kind, newParentId, index);
  const before = worldTransformFor(id, index, layoutById);
  const parentWorld = newParentId === null ? IDENTITY : worldTransformFor(newParentId, index, layoutById);
  const transform = relativeTransform(before, parentWorld);
  if (layout.kind === "section") return { ...layout, position: { x: transform.x, y: transform.y } };
  return { ...layout, transform };
}

function validateReparenting(
  id: StructureNodeId,
  kind: StructureLayout["kind"],
  newParentId: StructureNodeId | null,
  index: StructureIndex,
) {
  if (kind === "section" && newParentId !== null) throw new Error("A Section must remain at the Space root");
  if (newParentId === null) return;
  const parent = index.structureById.get(newParentId);
  if (!parent || parent.kind === "content" || newParentId === id)
    throw new Error(`Invalid structure parent: ${newParentId}`);
  if (ancestorsOf(newParentId, index).some((ancestor) => ancestor.id === id))
    throw new Error(`Structure cycle at: ${id}`);
}

function relativeTransform(before: LocalTransform, parentWorld: LocalTransform): LocalTransform {
  const radians = (parentWorld.rotation * Math.PI) / 180;
  const cos = Math.cos(radians);
  const sin = Math.sin(radians);
  const dx = before.x - parentWorld.x;
  const dy = before.y - parentWorld.y;
  const transform = {
    x: (dx * cos + dy * sin) / parentWorld.scale,
    y: (-dx * sin + dy * cos) / parentWorld.scale,
    rotation: before.rotation - parentWorld.rotation,
    scale: before.scale / parentWorld.scale,
  };
  assertTransform(transform);
  return transform;
}

export function assertTransform(transform: LocalTransform): void {
  if (
    !Number.isFinite(transform.x) ||
    !Number.isFinite(transform.y) ||
    !Number.isFinite(transform.rotation) ||
    !Number.isFinite(transform.scale) ||
    Math.abs(transform.x) > MAX_WORLD_COORDINATE ||
    Math.abs(transform.y) > MAX_WORLD_COORDINATE ||
    Math.abs(transform.rotation) > MAX_WORLD_COORDINATE ||
    transform.scale < 1 / MAX_WORLD_SCALE ||
    transform.scale > MAX_WORLD_SCALE
  ) {
    throw new Error("Invalid or degenerate canvas transform");
  }
}
