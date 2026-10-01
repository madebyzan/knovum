import type { SpaceRect } from "../contracts/generated/SpaceRect";
import type { SpaceEditSnapshot } from "../model/structure/schema/edit-snapshot";
import { createStructureIndex } from "../model/structure/structure-index";
import { boundsForOperationRoots } from "../model/structure/world-bounds";

const CONTAINMENT_TOLERANCE = 0.5;

/** Drawing considers complete root Content/Group units and never extracts Section descendants. */
export function sectionDrawCandidateIds(snapshot: SpaceEditSnapshot, frame: SpaceRect): string[] {
  const index = createStructureIndex(snapshot.structure);
  const layouts = new Map(snapshot.layouts.map((layout) => [layout.structureNodeId, layout]));
  return (index.childrenByParent.get(null) ?? []).flatMap((root) => {
    if (root.kind === "section") return [];
    const bounds = boundsForOperationRoots([root.id], index, layouts);
    return bounds &&
      bounds.left >= frame.x - CONTAINMENT_TOLERANCE &&
      bounds.top >= frame.y - CONTAINMENT_TOLERANCE &&
      bounds.right <= frame.x + frame.width + CONTAINMENT_TOLERANCE &&
      bounds.bottom <= frame.y + frame.height + CONTAINMENT_TOLERANCE
      ? [root.id]
      : [];
  });
}
