import type { StructureBounds } from "../../../canvas/contract/structure-layout";
import { composeStructureTransforms, invertStructureTransform } from "../../../canvas/document/structure-transform";
import { StructureError } from "../../../model/structure/structure-error";
import { structureSubtree } from "../../../model/structure/structure-index";
import { requireStructureLayout } from "../structure-context";
import { structureBoundsInParent } from "../structure-projection";
import { assertStructureEditable } from "../structure-selection";
import type { StructureCommandOf } from "./contract";
import { setDraftTransform } from "./draft";
import type { StructureOperation, StructureOperationResult } from "./operation";
import { sectionBoundsAround } from "./section-creation";

function requireSection(operation: StructureOperation, id: string) {
  const layout = requireStructureLayout(operation.context, id);
  if (layout.kind !== "section") throw new StructureError("invalid-selection", "This operation requires a Section", [id]);
  return layout;
}
function resizeFrame(operation: StructureOperation, id: string, bounds: StructureBounds) {
  const section = requireSection(operation, id);
  assertStructureEditable(operation.context.tree, [id]);
  if (section.collapsed) throw new StructureError("invalid-selection", "Expand the Section before resizing its region", [id]);
  const inverse = invertStructureTransform({ x: bounds.x, y: bounds.y, rotation: 0, scale: 1 });
  for (const child of operation.context.tree.children.get(id) ?? []) {
    setDraftTransform(operation.draft, child.id, composeStructureTransforms(inverse, operation.projection.transforms.get(child.id)!));
  }
  operation.draft.layouts.set(id, { ...section, position: { x: bounds.x, y: bounds.y }, size: { width: bounds.width, height: bounds.height } });
}
export function resizeSection(operation: StructureOperation, command: StructureCommandOf<"resize-section">): StructureOperationResult {
  resizeFrame(operation, command.id, command.bounds);
  return { selection: [command.id] };
}
export function fitSection(operation: StructureOperation, command: StructureCommandOf<"fit-section">): StructureOperationResult {
  requireSection(operation, command.id);
  const ids = (operation.context.tree.children.get(command.id) ?? []).map((item) => item.id);
  const bounds = structureBoundsInParent(operation.context, operation.projection, ids, null);
  if (!bounds) throw new StructureError("invalid-selection", "An empty Section has no content bounds", [command.id]);
  resizeFrame(operation, command.id, sectionBoundsAround(bounds));
  return { selection: [command.id] };
}
export function setSectionCollapsed(operation: StructureOperation, command: StructureCommandOf<"set-section-collapsed">): StructureOperationResult {
  const section = requireSection(operation, command.id);
  operation.draft.layouts.set(command.id, { ...section, collapsed: command.collapsed });
  if (!command.collapsed) return { selection: operation.selection };
  const hidden = new Set(structureSubtree(operation.context.tree, [command.id]).filter((item) => item.id !== command.id).map((item) => item.id));
  const selected = new Set(operation.selection.filter((id) => !hidden.has(id)));
  if (operation.selection.some((id) => hidden.has(id))) selected.add(command.id);
  return { selection: [...selected] };
}
