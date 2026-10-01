import type { SpaceRelation } from "../../model/relations/relation";
import type { StructureLayout, StructureNodeId } from "../../model/structure/schema/structure";
import { visibleRepresentativeForNode } from "../../model/structure/structure-queries";
import type { validateStructureSnapshot } from "../../model/structure/structure-validation";
import type { ProjectedRelation } from "./structure-scene";

interface StructureRelationProjectionInput {
  relations: readonly SpaceRelation[];
  index: ReturnType<typeof validateStructureSnapshot>;
  layouts: ReadonlyMap<StructureNodeId, StructureLayout>;
  temporarilyExpandedSectionIds: ReadonlySet<StructureNodeId>;
}

export function projectStructureRelations({
  relations,
  index,
  layouts,
  temporarilyExpandedSectionIds,
}: StructureRelationProjectionInput) {
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
