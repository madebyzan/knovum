import { computed, type MaybeRefOrGetter, toValue } from "vue";

import type { RelationKind } from "../../contracts/generated/RelationKind";
import type { RelationStatus } from "../../contracts/generated/RelationStatus";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceRelation } from "../../model/relations/relation";

interface VisibleCanvasContentInput {
  nodes: MaybeRefOrGetter<readonly SpaceCanvasNode[]>;
  relations: MaybeRefOrGetter<readonly SpaceRelation[]>;
  relationKinds: MaybeRefOrGetter<readonly RelationKind[]>;
  relationStatuses: MaybeRefOrGetter<readonly RelationStatus[]>;
}

export function useVisibleCanvasContent({
  nodes,
  relations,
  relationKinds,
  relationStatuses,
}: VisibleCanvasContentInput) {
  const visibleNodes = computed(() => toValue(nodes).filter((node) => !node.hidden));
  const visibleNodeIds = computed(() => new Set(visibleNodes.value.map(({ id }) => id)));
  const visibleRelations = computed(() => {
    const kinds = toValue(relationKinds);
    const statuses = toValue(relationStatuses);
    return toValue(relations).filter(
      (relation) =>
        visibleNodeIds.value.has(relation.sourceId) &&
        visibleNodeIds.value.has(relation.targetId) &&
        (!kinds.length || kinds.includes(relation.kind)) &&
        statuses.includes(relation.status),
    );
  });
  return { visibleNodes, visibleRelations };
}
