import type { SpaceRelation } from "../../../model/relations/relation";
import type { CanvasSceneNode } from "./canvas-scene";

export function parallelRelationOffsets(
  relations: readonly SpaceRelation[],
  nodes: ReadonlyMap<string, CanvasSceneNode>,
) {
  const groups = new Map<string, SpaceRelation[]>();
  for (const relation of relations) {
    const sourceId = nodes.get(relation.sourceId)?.node.id;
    const targetId = nodes.get(relation.targetId)?.node.id;
    if (!sourceId || !targetId || sourceId === targetId) continue;
    const key = [sourceId, targetId].toSorted().join("\u0000");
    const group = groups.get(key) ?? [];
    group.push(relation);
    groups.set(key, group);
  }
  const offsets = new Map<string, number>();
  for (const group of groups.values()) {
    const ordered = group.toSorted((left, right) => left.id.localeCompare(right.id));
    ordered.forEach((relation, index) => {
      const sourceId = nodes.get(relation.sourceId)?.node.id;
      const targetId = nodes.get(relation.targetId)?.node.id;
      if (!sourceId || !targetId) return;
      const direction = sourceId === [sourceId, targetId].toSorted()[0] ? 1 : -1;
      offsets.set(relation.id, (index - (ordered.length - 1) / 2) * 20 * direction);
    });
  }
  return offsets;
}
