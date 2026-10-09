import { compact, groupBy, sortBy } from "es-toolkit";

import type { SpaceCanvasNode } from "../../../model/nodes/canvas-node";
import type { SpaceRelation } from "../../../model/relations/relation";
import type { WorldBounds } from "../../../model/structure/world-bounds";
import { canvasNodeInteraction } from "../../canvas-node-interaction";
import { SECTION_HEADER_HEIGHT, type StructureScene } from "../../document/structure-scene";
import type { CanvasDocumentV1, CanvasNodeLayoutV1 } from "../../schema/document";
import type { CanvasScene, CanvasSceneNode, CanvasSceneRelation } from "./canvas-scene";
import { relationGeometry } from "./relation-path";

interface CanvasSceneProjectionInput {
  document: CanvasDocumentV1;
  nodes: readonly SpaceCanvasNode[];
  relations: readonly SpaceRelation[];
  lockedNodeIds?: ReadonlySet<string>;
  structure?: StructureScene;
}

export function projectCanvasScene({
  document,
  nodes,
  relations,
  lockedNodeIds: lockedIds,
  structure,
}: CanvasSceneProjectionInput): CanvasScene {
  const lockedNodeIds = lockedIds ?? new Set<string>();
  const nodeById = new Map(nodes.map((node) => [node.id, node]));
  const visibleStructureNodeIds = structure
    ? new Set(structure.content.filter(({ visible }) => visible).map(({ node }) => node.id))
    : undefined;
  const projected = sortBy(
    compact(
      document.nodes.map((layout): Omit<CanvasSceneNode, "paintOrder"> | undefined => {
        const node = nodeById.get(layout.nodeId);
        if (!node || node.hidden) return undefined;
        if (visibleStructureNodeIds && !visibleStructureNodeIds.has(node.id)) return undefined;
        return { interaction: canvasNodeInteraction(node, lockedNodeIds.has(node.id)), layout, node };
      }),
    ),
    [({ layout }) => layout.zIndex],
  );
  const paintOrderedNodes = projected.map((node, paintOrder) => Object.assign({}, node, { paintOrder }));
  const sceneNodeById = new Map(paintOrderedNodes.map((sceneNode) => [sceneNode.node.id, sceneNode]));
  const colorByRelationId = new Map(document.relationStyles?.map((style) => [style.relationId, style.color]));
  const relationEndpoints = structure
    ? structureRelationEndpoints(structure)
    : contentRelationEndpoints(projected, sceneNodeById);
  const projectedRelations =
    structure?.relations ??
    relations.map((relation) => ({
      relation,
      sourceStructureNodeId: relation.sourceId,
      targetStructureNodeId: relation.targetId,
    }));
  const relationOffsets = representativeRelationOffsets(projectedRelations);
  return {
    nodes: paintOrderedNodes,
    nodesById: sceneNodeById,
    sectionBackdrops:
      structure?.sections.map(({ bounds, section, layout, temporarilyExpanded }) => ({
        id: section.id,
        appearance: layout.appearance,
        bounds: {
          x: bounds.left,
          y: bounds.top,
          width: bounds.right - bounds.left,
          height:
            layout.collapsed && !temporarilyExpanded ? SECTION_HEADER_HEIGHT : bounds.bottom - bounds.top,
        },
      })) ?? [],
    relations: projectedRelations.flatMap(({ relation, sourceStructureNodeId, targetStructureNodeId }) =>
      projectRelation({
        relation,
        endpointByStructureNodeId: relationEndpoints,
        sourceStructureNodeId,
        targetStructureNodeId,
        color: colorByRelationId.get(relation.id) ?? "neutral",
        parallelOffset: relationOffsets.get(relation.id) ?? 0,
      }),
    ),
  };
}

interface CanvasRelationProjectionOptions {
  relation: SpaceRelation;
  endpointByStructureNodeId: ReadonlyMap<string, RelationEndpoint>;
  sourceStructureNodeId: string;
  targetStructureNodeId: string;
  color: CanvasSceneRelation["color"];
  parallelOffset: number;
}

function projectRelation(options: CanvasRelationProjectionOptions): CanvasSceneRelation[] {
  const {
    relation,
    endpointByStructureNodeId,
    sourceStructureNodeId,
    targetStructureNodeId,
    color,
    parallelOffset,
  } = options;
  const source = endpointByStructureNodeId.get(sourceStructureNodeId);
  const target = endpointByStructureNodeId.get(targetStructureNodeId);
  if (!source || !target || sourceStructureNodeId === targetStructureNodeId) return [];
  const geometry = relationGeometry(source.frame, target.frame, parallelOffset);
  return [
    {
      id: relation.id,
      key: `relation:${relation.id}`,
      kind: "relation",
      label: relation.kind,
      relation,
      color,
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
}

interface RelationEndpoint {
  frame: CanvasNodeLayoutV1["frame"];
  previewId: string;
}

function contentRelationEndpoints(
  projected: readonly Omit<CanvasSceneNode, "paintOrder">[],
  sceneNodeById: ReadonlyMap<string, CanvasSceneNode>,
) {
  return new Map(
    projected.flatMap(({ node }) => {
      const visible = sceneNodeById.get(node.id);
      return visible ? [[node.id, { frame: visible.layout.frame, previewId: node.id }] as const] : [];
    }),
  );
}

function structureRelationEndpoints(scene: StructureScene) {
  const endpointByStructureNodeId = new Map<string, RelationEndpoint>();
  for (const { bounds, structureNode } of scene.content)
    endpointByStructureNodeId.set(structureNode.id, {
      frame: boundsFrame(bounds),
      previewId: structureNode.nodeId,
    });
  for (const { bounds, section, layout, temporarilyExpanded } of scene.sections)
    endpointByStructureNodeId.set(section.id, {
      frame: boundsFrame(
        layout.collapsed && !temporarilyExpanded
          ? { ...bounds, bottom: bounds.top + SECTION_HEADER_HEIGHT }
          : bounds,
      ),
      previewId: `structure:${section.id}`,
    });
  return endpointByStructureNodeId;
}

function boundsFrame(bounds: WorldBounds) {
  return {
    x: bounds.left,
    y: bounds.top,
    width: bounds.right - bounds.left,
    height: bounds.bottom - bounds.top,
    rotation: 0,
  };
}

function representativeRelationOffsets(
  relations: readonly {
    relation: SpaceRelation;
    sourceStructureNodeId: string;
    targetStructureNodeId: string;
  }[],
) {
  const groups = groupBy(relations, (projected) =>
    [projected.sourceStructureNodeId, projected.targetStructureNodeId].toSorted().join("\u0000"),
  );
  const offsetByRelationId = new Map<string, number>();
  for (const group of Object.values(groups))
    group
      .toSorted((a, b) => a.relation.id.localeCompare(b.relation.id))
      .forEach((projected, index) => {
        const direction =
          projected.sourceStructureNodeId ===
          [projected.sourceStructureNodeId, projected.targetStructureNodeId].toSorted()[0]
            ? 1
            : -1;
        offsetByRelationId.set(projected.relation.id, (index - (group.length - 1) / 2) * 20 * direction);
      });
  return offsetByRelationId;
}
