import type {
  StructureBounds,
  StructureItemLayout,
  StructurePoint,
  StructureSize,
  StructureTransform,
} from "../contract/structure-layout";

export const IDENTITY_STRUCTURE_TRANSFORM: Readonly<StructureTransform> = Object.freeze({
  x: 0,
  y: 0,
  rotation: 0,
  scale: 1,
});
export function normalizeStructureRotation(degrees: number) {
  const wrapped = ((((degrees + 180) % 360) + 360) % 360) - 180;
  return Object.is(wrapped, -0) ? 0 : wrapped;
}
export function transformStructurePoint(
  transform: StructureTransform,
  point: StructurePoint,
): StructurePoint {
  const radians = (transform.rotation * Math.PI) / 180;
  const cosine = Math.cos(radians) * transform.scale;
  const sine = Math.sin(radians) * transform.scale;
  return {
    x: transform.x + cosine * point.x - sine * point.y,
    y: transform.y + sine * point.x + cosine * point.y,
  };
}
export function composeStructureTransforms(
  parent: StructureTransform,
  local: StructureTransform,
): StructureTransform {
  return {
    ...transformStructurePoint(parent, local),
    rotation: normalizeStructureRotation(parent.rotation + local.rotation),
    scale: parent.scale * local.scale,
  };
}
export function invertStructureTransform(transform: StructureTransform): StructureTransform {
  const linear = { x: 0, y: 0, rotation: -transform.rotation, scale: 1 / transform.scale };
  return { ...linear, ...transformStructurePoint(linear, { x: -transform.x, y: -transform.y }) };
}
export function structureLayoutTransform(layout: StructureItemLayout): StructureTransform {
  return layout.kind === "section" ? { ...layout.position, rotation: 0, scale: 1 } : layout.transform;
}
export function transformedStructureBounds(
  transform: StructureTransform,
  size: StructureSize,
): StructureBounds {
  const points = [
    { x: 0, y: 0 },
    { x: size.width, y: 0 },
    { x: 0, y: size.height },
    { x: size.width, y: size.height },
  ].map((point) => transformStructurePoint(transform, point));
  const x = Math.min(...points.map((point) => point.x));
  const y = Math.min(...points.map((point) => point.y));
  return {
    x,
    y,
    width: Math.max(...points.map((point) => point.x)) - x,
    height: Math.max(...points.map((point) => point.y)) - y,
  };
}
export function unionStructureBounds(bounds: readonly StructureBounds[]): StructureBounds | undefined {
  if (!bounds.length) return undefined;
  let left = Infinity,
    top = Infinity,
    right = -Infinity,
    bottom = -Infinity;
  for (const rect of bounds) {
    left = Math.min(left, rect.x);
    top = Math.min(top, rect.y);
    right = Math.max(right, rect.x + rect.width);
    bottom = Math.max(bottom, rect.y + rect.height);
  }
  return { x: left, y: top, width: right - left, height: bottom - top };
}
