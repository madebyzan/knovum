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

export function relativeTransform(world: LocalTransform, parent: LocalTransform): LocalTransform {
  const radians = (parent.rotation * Math.PI) / 180;
  const x = world.x - parent.x;
  const y = world.y - parent.y;
  return {
    x: (x * Math.cos(radians) + y * Math.sin(radians)) / parent.scale,
    y: (-x * Math.sin(radians) + y * Math.cos(radians)) / parent.scale,
    rotation: world.rotation - parent.rotation,
    scale: world.scale / parent.scale,
  };
}

export function worldTransformFor(
  structureNodeId: StructureNodeId,
  index: StructureIndex,
  layoutById: ReadonlyMap<StructureNodeId, StructureLayout>,
): LocalTransform {
  const structureNode = index.structureById.get(structureNodeId);
  if (!structureNode) throw new Error(`Structure structureNode not found: ${structureNodeId}`);
  let world = IDENTITY;
  for (const member of [...ancestorsOf(structureNodeId, index), structureNode]) {
    const layout = layoutById.get(member.id);
    if (!layout || layout.kind !== member.kind)
      throw new Error(`Structure layout missing or mismatched: ${member.id}`);
    world = composeTransforms(world, localTransformOf(layout));
  }
  return world;
}

function assertTransform(transform: LocalTransform): void {
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
