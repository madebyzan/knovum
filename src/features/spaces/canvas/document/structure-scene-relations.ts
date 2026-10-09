import type { SpaceRelation } from "../../model/relations/relation";
import type { StructureLayout, StructureNodeId } from "../../model/structure/schema/structure";
import type { StructureIndex } from "../../model/structure/structure-index";
import { visibleRepresentativeForNode } from "../../model/structure/structure-queries";
import type { ProjectedRelation } from "./structure-scene";

interface StructureRelationProjectionInput {
  relations: readonly SpaceRelation[];
  index: StructureIndex;
  layoutByStructureNodeId: ReadonlyMap<StructureNodeId, StructureLayout>;
  temporarilyExpandedSectionIds: ReadonlySet<StructureNodeId>;
}

export function projectStructureRelations({
  relations,
  index,
  layoutByStructureNodeId,
  temporarilyExpandedSectionIds,
}: StructureRelationProjectionInput) {
  return relations.flatMap<ProjectedRelation>((relation) => {
    const sourceStructureNodeId = visibleRepresentativeForNode(
      relation.sourceId,
      index,
      layoutByStructureNodeId,
      temporarilyExpandedSectionIds,
    );
    const targetStructureNodeId = visibleRepresentativeForNode(
      relation.targetId,
      index,
      layoutByStructureNodeId,
      temporarilyExpandedSectionIds,
    );
    return sourceStructureNodeId && targetStructureNodeId && sourceStructureNodeId !== targetStructureNodeId
      ? [{ relation, sourceStructureNodeId, targetStructureNodeId }]
      : [];
  });
}
