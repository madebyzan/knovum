import type { SpacePoint } from "../contracts/generated/SpacePoint";
import type { SpaceRect } from "../contracts/generated/SpaceRect";
import type { SpaceEditSnapshot } from "../model/structure/schema/edit-snapshot";
import { SECTION_DEFAULT_SIZE, SECTION_MINIMUM_SIZE } from "../model/structure/section-metrics";
import { createStructureIndex } from "../model/structure/structure-index";
import { boundsForOperationRoots } from "../model/structure/world-bounds";

const CONTAINMENT_TOLERANCE = 0.5;

/** Drawing considers complete root Content/Group units and never extracts Section descendants. */
export function sectionDrawCandidateIds(snapshot: SpaceEditSnapshot, frame: SpaceRect): string[] {
  const index = createStructureIndex(snapshot.structure);
  const layoutByStructureNodeId = new Map(snapshot.layouts.map((layout) => [layout.structureNodeId, layout]));
  return (index.childrenByParent.get(null) ?? []).flatMap((root) => {
    if (root.kind === "section") return [];
    const bounds = boundsForOperationRoots([root.id], index, layoutByStructureNodeId);
    return bounds &&
      bounds.left >= frame.x - CONTAINMENT_TOLERANCE &&
      bounds.top >= frame.y - CONTAINMENT_TOLERANCE &&
      bounds.right <= frame.x + frame.width + CONTAINMENT_TOLERANCE &&
      bounds.bottom <= frame.y + frame.height + CONTAINMENT_TOLERANCE
      ? [root.id]
      : [];
  });
}

export function sectionDrawFrame(origin: SpacePoint, point: SpacePoint): SpaceRect {
  const dx = point.x - origin.x;
  const dy = point.y - origin.y;
  if (Math.hypot(dx, dy) < 4) return { ...origin, ...SECTION_DEFAULT_SIZE };
  const width = Math.max(Math.abs(dx), SECTION_MINIMUM_SIZE.width);
  const height = Math.max(Math.abs(dy), SECTION_MINIMUM_SIZE.height);
  return {
    x: dx < 0 ? origin.x - width : origin.x,
    y: dy < 0 ? origin.y - height : origin.y,
    width,
    height,
  };
}
