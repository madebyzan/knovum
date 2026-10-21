import type { SpaceRelation } from "./relation";

/**
 * Lineage is the history a derivation records: one `derives-from` relation from each output to
 * the content it was made from. It explains where content came from rather than stating what it
 * means, and a single source can carry many outputs, so the canvas draws it on demand instead of
 * at rest.
 */
export function isLineageRelation(relation: Pick<SpaceRelation, "kind">) {
  return relation.kind === "derives-from";
}

/**
 * The lineage relations that explain `nodeIds`: every derivation step from each node up to the
 * content it ultimately came from, and down to everything derived from it. Siblings stay out —
 * they share a source with the node, not its history.
 */
export function lineageRelationIds(
  relations: readonly SpaceRelation[],
  nodeIds: Iterable<string>,
): ReadonlySet<string> {
  const toSources = new Map<string, SpaceRelation[]>();
  const toOutputs = new Map<string, SpaceRelation[]>();
  for (const relation of relations) {
    if (!isLineageRelation(relation)) continue;
    append(toSources, relation.sourceId, relation);
    append(toOutputs, relation.targetId, relation);
  }
  const seeds = [...nodeIds];
  const revealed = new Set<string>();
  follow(seeds, toSources, ({ targetId }) => targetId, revealed);
  follow(seeds, toOutputs, ({ sourceId }) => sourceId, revealed);
  return revealed;
}

/** How many distinct outputs derive directly from each node. */
export function derivedOutputCounts(relations: readonly SpaceRelation[]): ReadonlyMap<string, number> {
  const outputsBySource = new Map<string, Set<string>>();
  for (const relation of relations) {
    if (!isLineageRelation(relation)) continue;
    const outputs = outputsBySource.get(relation.targetId) ?? new Set<string>();
    outputs.add(relation.sourceId);
    outputsBySource.set(relation.targetId, outputs);
  }
  return new Map([...outputsBySource].map(([source, outputs]) => [source, outputs.size]));
}

function follow(
  seeds: readonly string[],
  edges: ReadonlyMap<string, readonly SpaceRelation[]>,
  next: (relation: SpaceRelation) => string,
  revealed: Set<string>,
) {
  const visited = new Set(seeds);
  const queue = [...visited];
  for (const nodeId of queue) {
    for (const relation of edges.get(nodeId) ?? []) {
      revealed.add(relation.id);
      const reached = next(relation);
      if (visited.has(reached)) continue;
      visited.add(reached);
      queue.push(reached);
    }
  }
}

function append(edges: Map<string, SpaceRelation[]>, nodeId: string, relation: SpaceRelation) {
  const list = edges.get(nodeId) ?? [];
  list.push(relation);
  edges.set(nodeId, list);
}
