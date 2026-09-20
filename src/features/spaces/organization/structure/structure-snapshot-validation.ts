import {
  WORLD_COORDINATE_LIMIT,
  WORLD_SCALE_MAX,
  WORLD_SCALE_MIN,
} from "../../canvas/contract/structure-layout";
import { StructureError } from "../../model/structure/structure-error";
import { validateContentPlacements, validateStructure } from "../../model/structure/structure-validation";
import { indexStructureSnapshot } from "./structure-context";
import { projectStructure } from "./structure-projection";
import { structureSnapshotSchema, type StructureSnapshot } from "./structure-snapshot-contract";

export function parseStructureSnapshot(value: unknown, contentIds: ReadonlySet<string>): StructureSnapshot {
  const snapshot = structureSnapshotSchema.parse(value);
  const context = validateStructureSnapshot(snapshot);
  validateContentPlacements(context.tree, contentIds);
  return snapshot;
}
export function validateStructureSnapshot(snapshot: StructureSnapshot) {
  const context = indexStructureSnapshot(snapshot);
  validateStructure(context.tree);
  const projection = projectStructure(context);
  for (const [id, transform] of projection.transforms) {
    if (
      !validCoordinate(transform.x) ||
      !validCoordinate(transform.y) ||
      transform.scale < WORLD_SCALE_MIN ||
      transform.scale > WORLD_SCALE_MAX
    ) {
      throw new StructureError("invalid-layout", "Composed world transforms must be bounded and invertible", [
        id,
      ]);
    }
  }
  for (const [id, bounds] of projection.bounds) {
    if (
      bounds.width <= 0 ||
      bounds.height <= 0 ||
      ![bounds.x, bounds.y, bounds.x + bounds.width, bounds.y + bounds.height].every(validCoordinate)
    ) {
      throw new StructureError("invalid-layout", "World bounds must remain finite and non-degenerate", [id]);
    }
  }
  return context;
}
function validCoordinate(value: number) {
  return Number.isFinite(value) && Math.abs(value) <= WORLD_COORDINATE_LIMIT;
}
