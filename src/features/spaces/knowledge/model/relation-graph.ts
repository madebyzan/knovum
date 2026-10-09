import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import {
  isSymmetricRelationKind,
  type SpaceRelation,
  type SpaceRelationStatus,
} from "../../model/relations/relation";
import type { SpaceSavedViewFilter } from "./saved-view-filter";
import { isSpaceSemanticNode } from "./semantic-node";

export function relationNeighborhood(
  relations: readonly SpaceRelation[],
  seeds: readonly string[],
  depth: number,
  statuses: readonly SpaceRelationStatus[] = ["active", "contested"],
) {
  const visited = new Set(seeds);
  let frontier = [...seeds];
  for (let step = 0; step < depth && frontier.length; step += 1) {
    const next: string[] = [];
    for (const relation of relations) {
      if (!statuses.includes(relation.status)) continue;
      const neighbor = frontier.includes(relation.sourceId)
        ? relation.targetId
        : frontier.includes(relation.targetId)
          ? relation.sourceId
          : undefined;
      if (neighbor && !visited.has(neighbor)) {
        visited.add(neighbor);
        next.push(neighbor);
      }
    }
    frontier = next;
  }
  return visited;
}

export function findSemanticRelationPath(
  relations: readonly SpaceRelation[],
  sourceId: string,
  targetId: string,
  maxDepth = 6,
): string[] {
  if (sourceId === targetId) return [sourceId];
  const previousNodeIdByNodeId = new Map<string, string>();
  const depthByNodeId = new Map([[sourceId, 0]]);
  const queue = [sourceId];
  while (queue.length) {
    const current = queue.shift();
    if (!current) break;
    const currentDepth = depthByNodeId.get(current) ?? 0;
    if (currentDepth >= maxDepth) continue;
    for (const neighbor of connectedIds(relations, current)) {
      if (depthByNodeId.has(neighbor)) continue;
      previousNodeIdByNodeId.set(neighbor, current);
      depthByNodeId.set(neighbor, currentDepth + 1);
      if (neighbor === targetId) return unwindPath(previousNodeIdByNodeId, sourceId, targetId);
      queue.push(neighbor);
    }
  }
  return [];
}

export type SpaceRelationIndex = ReadonlyMap<string, readonly SpaceRelation[]>;

function createSpaceRelationIndex(relations: readonly SpaceRelation[]): SpaceRelationIndex {
  const relationsByNodeId = new Map<string, SpaceRelation[]>();
  for (const relation of relations) {
    appendIndexedRelation(relationsByNodeId, relation.sourceId, relation);
    appendIndexedRelation(relationsByNodeId, relation.targetId, relation);
  }
  return relationsByNodeId;
}

interface MatchingNodesQuery {
  nodes: readonly SpaceCanvasNode[];
  relations: readonly SpaceRelation[];
  filter: SpaceSavedViewFilter;
  index?: SpaceRelationIndex;
}

export function queryMatchingNodes({
  nodes,
  relations,
  filter,
  index = createSpaceRelationIndex(relations),
}: MatchingNodesQuery) {
  return nodes.filter((node) => matchesNode(node, index, filter));
}

function appendIndexedRelation(
  relationsByNodeId: Map<string, SpaceRelation[]>,
  nodeId: string,
  relation: SpaceRelation,
) {
  const bucket = relationsByNodeId.get(nodeId);
  if (bucket) bucket.push(relation);
  else relationsByNodeId.set(nodeId, [relation]);
}

function connectedIds(relations: readonly SpaceRelation[], nodeId: string) {
  const ids: string[] = [];
  for (const relation of relations) {
    if (relation.status === "invalidated") continue;
    if (relation.sourceId === nodeId) ids.push(relation.targetId);
    else if (isSymmetricRelationKind(relation.kind) && relation.targetId === nodeId)
      ids.push(relation.sourceId);
  }
  return ids;
}

function unwindPath(previousNodeIdByNodeId: ReadonlyMap<string, string>, sourceId: string, targetId: string) {
  const path = [targetId];
  while (path[0] !== sourceId) {
    const prior = previousNodeIdByNodeId.get(path[0] ?? "");
    if (!prior) return [];
    path.unshift(prior);
  }
  return path;
}

function matchesNode(node: SpaceCanvasNode, relationIndex: SpaceRelationIndex, filter: SpaceSavedViewFilter) {
  if (!isSpaceSemanticNode(node)) return false;
  if (!matchesTextAndTags(node, filter)) return false;
  if (!matchesObjectFilters(node, filter)) return false;
  return matchesRelationFilters(node.id, relationIndex, filter);
}

function matchesTextAndTags(node: SpaceCanvasNode, filter: SpaceSavedViewFilter) {
  if (filter.text && !node.data.title.toLocaleLowerCase().includes(filter.text.toLocaleLowerCase()))
    return false;
  if (!filter.tags?.length) return true;
  return filter.tags.every((tag) => node.data.tags?.includes(tag));
}

function matchesObjectFilters(node: SpaceCanvasNode, filter: SpaceSavedViewFilter) {
  if (filter.objectTypeIds?.length) {
    if (node.data.kind !== "object" || !filter.objectTypeIds.includes(node.data.objectTypeId)) return false;
  }
  if (!filter.statuses?.length) return true;
  return node.data.kind === "object" && filter.statuses.includes(node.data.status);
}

function matchesRelationFilters(
  nodeId: string,
  relationIndex: SpaceRelationIndex,
  filter: SpaceSavedViewFilter,
) {
  if (!filter.relationKinds?.length && !filter.relationStatuses?.length) return true;
  return (relationIndex.get(nodeId) ?? []).some((relation) =>
    relationMatchesFilter(nodeId, relation, filter),
  );
}

function relationMatchesFilter(nodeId: string, relation: SpaceRelation, filter: SpaceSavedViewFilter) {
  if (relation.sourceId !== nodeId && relation.targetId !== nodeId) return false;
  if (filter.relationKinds?.length && !filter.relationKinds.includes(relation.kind)) return false;
  return !filter.relationStatuses?.length || filter.relationStatuses.includes(relation.status);
}
