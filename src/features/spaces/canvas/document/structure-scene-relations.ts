import type { SpaceRelation } from "../../model/relations/relation";
import type { StructureLayout, StructureNodeId } from "../../model/structure/structure-contract";
import { visibleRepresentativeForNode } from "../../model/structure/structure-queries";
import type { validateStructureSnapshot } from "../../model/structure/structure-validation";
import type { ProjectedRelation } from "./structure-scene";

export function projectStructureRelations(
  relations: readonly SpaceRelation[],
  index: ReturnType<typeof validateStructureSnapshot>,
  layouts: ReadonlyMap<StructureNodeId, StructureLayout>,
  temporarilyExpandedSectionIds: ReadonlySet<StructureNodeId>,
) {
  return relations.flatMap<ProjectedRelation>((relation) => {
    const sourceStructureNodeId = visibleRepresentativeForNode(
      relation.sourceId,
      index,
      layouts,
      temporarilyExpandedSectionIds,
    );
    const targetStructureNodeId = visibleRepresentativeForNode(
      relation.targetId,
      index,
      layouts,
      temporarilyExpandedSectionIds,
    );
    return sourceStructureNodeId && targetStructureNodeId && sourceStructureNodeId !== targetStructureNodeId
      ? [{ relation, sourceStructureNodeId, targetStructureNodeId }]
      : [];
  });
}
