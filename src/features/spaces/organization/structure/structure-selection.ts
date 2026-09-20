import { StructureError } from "../../model/structure/structure-error";
import {
  requireStructureItem,
  structureAncestors,
  structureSubtree,
  type StructureIndex,
} from "../../model/structure/structure-index";

export function structureOperationRoots(index: StructureIndex, ids: readonly string[]) {
  const selected = new Set(ids);
  return [...selected]
    .map((id) => requireStructureItem(index, id))
    .filter((item) => !structureAncestors(index, item.id).some((parent) => selected.has(parent.id)));
}
export function requireStructureRoots(index: StructureIndex, ids: readonly string[]) {
  const roots = structureOperationRoots(index, ids);
  if (!roots.length) throw new StructureError("invalid-selection", "The operation requires a selection");
  return roots;
}
export function assertStructureEditable(index: StructureIndex, ids: readonly string[]) {
  const blocked = ids.filter(
    (id) =>
      requireStructureItem(index, id).locked || structureAncestors(index, id).some((parent) => parent.locked),
  );
  if (blocked.length)
    throw new StructureError("locked", "The operation is blocked by locked items or ancestors", blocked);
}
export function structureContentIds(index: StructureIndex, ids: readonly string[]) {
  return structureSubtree(
    index,
    structureOperationRoots(index, ids).map((item) => item.id),
  ).flatMap((item) => (item.kind === "content" ? [item.nodeId] : []));
}
export function containingStructureSection(index: StructureIndex, id: string) {
  const item = requireStructureItem(index, id);
  return item.kind === "section"
    ? item.id
    : structureAncestors(index, id).find((parent) => parent.kind === "section")?.id;
}
export function structureSelectionTarget(
  index: StructureIndex,
  id: string,
  scope: string | null,
): string | undefined {
  const item = requireStructureItem(index, id);
  const ancestors = structureAncestors(index, id);
  if (scope !== null) {
    if (requireStructureItem(index, scope).kind !== "group")
      throw new StructureError("invalid-selection", "Only Groups define an editing scope", [scope]);
    if (id === scope) return id;
    const scopeIndex = ancestors.findIndex((parent) => parent.id === scope);
    if (scopeIndex < 0) return undefined;
    ancestors.length = scopeIndex;
  }
  return ancestors.findLast((parent) => parent.kind === "group")?.id ?? item.id;
}
