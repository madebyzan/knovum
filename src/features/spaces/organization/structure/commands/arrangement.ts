import type { StructureBounds } from "../../../canvas/contract/structure-layout";
import { unionStructureBounds } from "../../../canvas/document/structure-transform";
import { StructureError } from "../../../model/structure/structure-error";
import { assertStructureEditable } from "../structure-selection";
import type { StructureCommandOf } from "./contract";
import { moveRoot } from "./movement";
import { operationRoots, type StructureOperation, type StructureOperationResult } from "./operation";

export function alignItems(operation: StructureOperation, command: StructureCommandOf<"align-items">): StructureOperationResult {
  const roots = arrangementRoots(operation, command.ids, 2);
  const bounds = unionStructureBounds(roots.map((item) => operation.projection.bounds.get(item.id)!))!;
  const fraction = { start: 0, center: 0.5, end: 1 }[command.alignment];
  const target = anchor(bounds, command.axis, fraction);
  for (const item of roots) {
    const offset = target - anchor(operation.projection.bounds.get(item.id)!, command.axis, fraction);
    moveRoot(operation, item, command.axis === "x" ? { x: offset, y: 0 } : { x: 0, y: offset });
  }
  return { selection: roots.map((item) => item.id) };
}
export function distributeItems(operation: StructureOperation, command: StructureCommandOf<"distribute-items">): StructureOperationResult {
  const { axis } = command;
  const size = axis === "x" ? "width" : "height";
  const roots = arrangementRoots(operation, command.ids, 3).toSorted((a, b) => operation.projection.bounds.get(a.id)![axis] - operation.projection.bounds.get(b.id)![axis] || operation.projection.paintOrder.get(a.id)! - operation.projection.paintOrder.get(b.id)!);
  const bounds = roots.map((item) => operation.projection.bounds.get(item.id)!);
  const first = bounds[0]!; const last = bounds[bounds.length - 1]!;
  const span = last[axis] + last[size] - first[axis];
  const gap = (span - bounds.reduce((sum, rect) => sum + rect[size], 0)) / (roots.length - 1);
  let position = first[axis];
  for (const [index, item] of roots.entries()) {
    const rect = bounds[index]!;
    const offset = index === 0 || index === roots.length - 1 ? 0 : position - rect[axis];
    moveRoot(operation, item, axis === "x" ? { x: offset, y: 0 } : { x: 0, y: offset });
    position += rect[size] + gap;
  }
  return { selection: roots.map((item) => item.id) };
}
function arrangementRoots(operation: StructureOperation, ids: readonly string[], count: number) {
  const roots = operationRoots(operation, ids);
  if (roots.length < count) throw new StructureError("invalid-selection", `This arrangement requires ${count} operation roots`);
  assertStructureEditable(operation.context.tree, roots.map((item) => item.id));
  return roots;
}
function anchor(bounds: StructureBounds, axis: "x" | "y", fraction: number) { return bounds[axis] + bounds[axis === "x" ? "width" : "height"] * fraction; }
