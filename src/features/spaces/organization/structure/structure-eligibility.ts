import { indexStructure, structureAncestors } from "../../model/structure/structure-index";
import type { SpaceStructure } from "../../model/structure/structure-schema";
import { structureOperationRoots } from "./structure-selection";
/** A single capability query is shared by menus, shortcuts, toolbar and Outline. */
export function structureSelectionEligibility(structure: SpaceStructure, ids: readonly string[]) {
  const tree = indexStructure(structure);
  const roots = structureOperationRoots(
    tree,
    ids.filter((id) => tree.items.has(id)),
  );
  const editable =
    roots.length > 0 &&
    roots.every(
      (item) => !item.locked && structureAncestors(tree, item.id).every((parent) => !parent.locked),
    );
  const sameParent = roots.length > 0 && roots.every((item) => item.parentId === roots[0]!.parentId);
  const parent = roots[0]?.parentId;
  const sectionScope =
    parent === null || (parent !== undefined && tree.items.get(parent)?.kind === "section");
  const noSections = roots.every((item) => item.kind !== "section");
  return {
    roots,
    editable,
    sameParent,
    canGroup: editable && sameParent && noSections && roots.length >= 2,
    canSection: editable && sameParent && noSections && sectionScope,
    canUngroup: editable && roots.length === 1 && roots[0]?.kind === "group",
  };
}
