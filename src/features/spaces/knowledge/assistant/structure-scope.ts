import { indexStructure } from "../../model/structure/structure-index";
import type { SpaceStructure } from "../../model/structure/structure-schema";
import {
  containingStructureSection,
  structureContentIds,
  structureOperationRoots,
} from "../../organization/structure/structure-selection";

/** Knowledge scope follows the complete structure, never its canvas visibility. */
export function resolveStructureKnowledgeScope(
  structure: SpaceStructure,
  selectedItemIds: readonly string[],
  eligibleNodeIds: ReadonlySet<string>,
) {
  const tree = indexStructure(structure);
  // Selection is transient and can briefly retain identities removed by a committed edit.
  const available = selectedItemIds.filter((id) => tree.items.has(id));
  const roots = structureOperationRoots(tree, available).map((item) => item.id);
  const eligible = (ids: readonly string[]) => ids.filter((id) => eligibleNodeIds.has(id));
  const selection = eligible(structureContentIds(tree, roots));
  const sections = new Set(roots.map((id) => containingStructureSection(tree, id)));
  const sectionId = sections.size === 1 ? sections.values().next().value : undefined;
  const section = sectionId ? eligible(structureContentIds(tree, [sectionId])) : [];
  return { selection, section, sectionId };
}
