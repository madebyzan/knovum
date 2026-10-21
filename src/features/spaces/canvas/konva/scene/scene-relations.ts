import { groupBy } from "es-toolkit";

import { isLineageRelation } from "../../../model/relations/lineage";
import { canonicalRelationEndpoints, type SpaceRelation } from "../../../model/relations/relation";
import type { CanvasNodeLayoutV1, CanvasRelationColor } from "../../schema/document";
import type { CanvasSceneRelation } from "./canvas-scene";
import { relationGeometry } from "./relation-path";

/** Where a relation attaches: the frame drawn for a visible structure node. */
export interface RelationEndpoint {
  frame: CanvasNodeLayoutV1["frame"];
  previewId: string;
}

/** A relation and the visible structure nodes that stand for its two endpoints. */
export interface PlacedRelation {
  relation: SpaceRelation;
  sourceStructureNodeId: string;
  targetStructureNodeId: string;
}

/** Relations of one kind between the same visible endpoints, drawn as one line. */
interface RelationBundle {
  id: string;
  kind: SpaceRelation["kind"];
  relations: SpaceRelation[];
  source: string;
  target: string;
}

/** Spacing between lines that join the same two endpoints, so each stays legible. */
const PARALLEL_SPACING = 20;

export function projectSceneRelations(
  placed: readonly PlacedRelation[],
  endpoints: ReadonlyMap<string, RelationEndpoint>,
  colorByRelationId: ReadonlyMap<string, CanvasRelationColor>,
): CanvasSceneRelation[] {
  const bundles = bundleRelations(placed);
  const offsets = parallelOffsets(bundles);
  return bundles.flatMap((bundle) => {
    const source = endpoints.get(bundle.source);
    const target = endpoints.get(bundle.target);
    if (!source || !target || bundle.source === bundle.target) return [];
    const parallelOffset = offsets.get(bundle.id) ?? 0;
    const geometry = relationGeometry(source.frame, target.frame, parallelOffset);
    return [
      {
        id: bundle.id,
        key: `relation:${bundle.id}`,
        kind: "relation",
        label: bundle.kind,
        lineage: isLineageRelation(bundle),
        relations: bundle.relations,
        color: bundleColor(bundle, colorByRelationId),
        bounds: geometry.bounds,
        midpoint: geometry.midpoint,
        points: geometry.points,
        sourceNodeId: source.previewId,
        sourceFrame: source.frame,
        targetNodeId: target.previewId,
        targetFrame: target.frame,
        parallelOffset,
      },
    ];
  });
}

function bundleRelations(placed: readonly PlacedRelation[]): RelationBundle[] {
  const bundles = new Map<string, Omit<RelationBundle, "id">>();
  for (const { relation, sourceStructureNodeId, targetStructureNodeId } of placed) {
    const [source, target] = canonicalRelationEndpoints(
      sourceStructureNodeId,
      targetStructureNodeId,
      relation.kind,
    );
    const key = `${relation.kind}:${source}:${target}`;
    const bundle = bundles.get(key);
    if (bundle) bundle.relations.push(relation);
    else bundles.set(key, { kind: relation.kind, relations: [relation], source, target });
  }
  // A line that stands for one relation is that relation, so selecting and styling it stay direct.
  return [...bundles].map(([key, bundle]) =>
    Object.assign(bundle, {
      id: bundle.relations.length === 1 ? bundle.relations[0]!.id : `bundle:${key}`,
    }),
  );
}

/** One colour when every relation in the line agrees on it, the neutral colour otherwise. */
function bundleColor(
  bundle: RelationBundle,
  colorByRelationId: ReadonlyMap<string, CanvasRelationColor>,
): CanvasRelationColor {
  const colors = new Set(bundle.relations.map(({ id }) => colorByRelationId.get(id) ?? "neutral"));
  return colors.size === 1 ? [...colors][0]! : "neutral";
}

function parallelOffsets(bundles: readonly RelationBundle[]) {
  const pairKey = ({ source, target }: RelationBundle) => [source, target].toSorted().join("\u0000");
  const offsetById = new Map<string, number>();
  for (const pair of Object.values(groupBy(bundles, pairKey)))
    pair
      .toSorted((left, right) => left.id.localeCompare(right.id))
      .forEach((bundle, index) => {
        const direction = bundle.source === [bundle.source, bundle.target].toSorted()[0] ? 1 : -1;
        offsetById.set(bundle.id, (index - (pair.length - 1) / 2) * PARALLEL_SPACING * direction);
      });
  return offsetById;
}

/** The lines a canvas shows when nothing is selected: lineage is drawn only on demand. */
export function restingRelations(relations: readonly CanvasSceneRelation[]) {
  return relations.filter(({ lineage }) => !lineage);
}
