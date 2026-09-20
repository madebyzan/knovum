import type { StructureBounds, StructureTransform } from "../../canvas/contract/structure-layout";
import {
  composeStructureTransforms,
  IDENTITY_STRUCTURE_TRANSFORM,
  invertStructureTransform,
  structureLayoutTransform,
  transformedStructureBounds,
  unionStructureBounds,
} from "../../canvas/document/structure-transform";
import { requireStructureItem, structureSubtree } from "../../model/structure/structure-index";
import { requireStructureLayout, type StructureContext } from "./structure-context";

export interface StructureProjection {
  readonly transforms: ReadonlyMap<string, StructureTransform>;
  readonly bounds: ReadonlyMap<string, StructureBounds>;
  readonly paintOrder: ReadonlyMap<string, number>;
}
export function projectStructure(context: StructureContext): StructureProjection {
  const ordered = structureSubtree(
    context.tree,
    (context.tree.children.get(null) ?? []).map((item) => item.id),
  );
  const transforms = new Map<string, StructureTransform>();
  const bounds = new Map<string, StructureBounds>();
  const paintOrder = new Map<string, number>();
  for (const [order, item] of ordered.entries()) {
    const layout = requireStructureLayout(context, item.id);
    const parent = item.parentId === null ? IDENTITY_STRUCTURE_TRANSFORM : transforms.get(item.parentId)!;
    const world = composeStructureTransforms(parent, structureLayoutTransform(layout));
    transforms.set(item.id, world);
    paintOrder.set(item.id, order);
    if (layout.kind !== "group") bounds.set(item.id, transformedStructureBounds(world, layout.size));
  }
  for (const item of ordered.toReversed()) {
    if (item.kind !== "group") continue;
    bounds.set(
      item.id,
      unionStructureBounds((context.tree.children.get(item.id) ?? []).map((child) => bounds.get(child.id)!))!,
    );
  }
  return { transforms, bounds, paintOrder };
}
/** Tight bounds in the requested parent's coordinate system, not a re-rotated world AABB. */
export function structureBoundsInParent(
  context: StructureContext,
  projection: StructureProjection,
  ids: readonly string[],
  parentId: string | null,
): StructureBounds | undefined {
  const inverse = invertStructureTransform(
    parentId === null
      ? IDENTITY_STRUCTURE_TRANSFORM
      : projection.transforms.get(requireStructureItem(context.tree, parentId).id)!,
  );
  const pending = [...ids];
  const seen = new Set<string>();
  const bounds: StructureBounds[] = [];
  while (pending.length) {
    const id = pending.pop()!;
    if (seen.has(id)) continue;
    seen.add(id);
    const layout = requireStructureLayout(context, id);
    if (layout.kind === "group") {
      for (const child of context.tree.children.get(id) ?? []) pending.push(child.id);
    } else
      bounds.push(
        transformedStructureBounds(
          composeStructureTransforms(inverse, projection.transforms.get(id)!),
          layout.size,
        ),
      );
  }
  return unionStructureBounds(bounds);
}
