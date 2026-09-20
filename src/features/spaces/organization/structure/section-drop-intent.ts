import type { StructurePoint } from "../../canvas/contract/structure-layout";
import { indexStructureSnapshot } from "./structure-context";
import { assertStructureEditable, structureOperationRoots } from "./structure-selection";
import type { StructureSnapshot } from "./structure-snapshot-contract";
export function resolveSectionDrop(
  snapshot: StructureSnapshot,
  ids: readonly string[],
  point: StructurePoint,
  previous: string | null | undefined,
  zoom: number,
): string | null | undefined {
  const context = indexStructureSnapshot(snapshot);
  const roots = structureOperationRoots(context.tree, ids);
  assertStructureEditable(
    context.tree,
    roots.map((item) => item.id),
  );
  if (
    roots.some(
      (item) =>
        item.kind === "section" ||
        (item.parentId !== null && context.tree.items.get(item.parentId)?.kind === "group"),
    )
  )
    return undefined;
  const candidates = (context.tree.children.get(null) ?? [])
    .filter((item) => item.kind === "section" && !item.locked)
    .toSorted((a, b) => b.order - a.order);
  const contains = (id: string, margin: number) => {
    const layout = context.layouts.get(id)!;
    return (
      layout.kind === "section" &&
      !layout.collapsed &&
      point.x >= layout.position.x - margin &&
      point.x <= layout.position.x + layout.size.width + margin &&
      point.y >= layout.position.y - margin &&
      point.y <= layout.position.y + layout.size.height + margin
    );
  };
  const direct = candidates.find((item) => contains(item.id, 0));
  if (direct) return direct.id;
  if (previous && candidates.some((item) => item.id === previous) && contains(previous, 8 / zoom))
    return previous;
  return null;
}
