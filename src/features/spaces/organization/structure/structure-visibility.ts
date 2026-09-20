import { requireStructureItem, structureAncestors } from "../../model/structure/structure-index";
import type { StructureContext } from "./structure-context";
export function visibleStructureRepresentative(context: StructureContext, id: string): string {
  requireStructureItem(context.tree, id);
  const section = structureAncestors(context.tree, id).find((item) => item.kind === "section");
  if (!section) return id;
  const layout = context.layouts.get(section.id)!;
  return layout.kind === "section" && layout.collapsed ? section.id : id;
}
