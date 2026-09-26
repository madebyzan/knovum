import { sortBy } from "es-toolkit";

import type { SpaceCanvasNode } from "../../../model/nodes/canvas-node";
import type { SpaceRelation } from "../../../model/relations/relation";
import { canvasNodeInteraction } from "../../canvas-node-interaction";
import type { CanvasDocumentV1, CanvasNodeLayoutV1 } from "../../contract/canvas-document";
import type { StructureScene } from "../../document/structure-scene";
import type { CanvasScene, CanvasSceneNode, CanvasSceneRelation } from "./canvas-scene";
import { relationGeometry } from "./relation-path";

export type { CanvasScene, CanvasSceneNode, CanvasSceneRelation } from "./canvas-scene";

export function projectCanvasScene(
  document: CanvasDocumentV1,
  nodes: readonly SpaceCanvasNode[],
  relations: readonly SpaceRelation[],
  options: { lockedNodeIds?: ReadonlySet<string>; structure?: StructureScene } = {},
): CanvasScene {
  const lockedNodeIds = options.lockedNodeIds ?? new Set<string>();
  const structure = options.structure;
  const content = new Map(nodes.map((node) => [node.id, node]));
  const visibleStructureNodeIds = structure
    ? new Set(structure.content.filter(({ visible }) => visible).map(({ node }) => node.id))
    : undefined;
  const projected = sortBy(
    document.nodes.flatMap<Omit<CanvasSceneNode, "paintOrder">>((layout) => {
      const node = content.get(layout.nodeId);
      return node && !node.hidden && (!visibleStructureNodeIds || visibleStructureNodeIds.has(node.id))
        ? [
            {
              interaction: canvasNodeInteraction(node, lockedNodeIds.has(node.id)),
              layout,
              node,
            },
          ]
        : [];
    }),
    [({ layout }) => layout.zIndex],
  );
  const paintOrderedNodes = projected.map((node, paintOrder) => Object.assign({}, node, { paintOrder }));
  const byId = new Map(paintOrderedNodes.map((sceneNode) => [sceneNode.node.id, sceneNode]));
  const relationStyles = new Map(document.relationStyles?.map((style) => [style.relationId, style.color]));
  const relationEndpoints = structure
    ? structureRelationEndpoints(structure)
    : contentRelationEndpoints(projected, byId);
  const projectedRelations =
    structure?.relations ??
    relations.map((relation) => ({
      relation,
      sourceStructureNodeId: relation.sourceId,
      targetStructureNodeId: relation.targetId,
    }));
  const relationOffsets = representativeRelationOffsets(projectedRelations);
  return {
    contentNodes: paintOrderedNodes,
    nodes: paintOrderedNodes,
    nodesById: byId,
    sectionBackdrops:
      structure?.sections.map(({ bounds, section, layout, temporarilyExpanded }) => ({
        id: section.id,
        appearance: layout.appearance,
        bounds: {
          x: bounds.left,
          y: bounds.top,
          width: bounds.right - bounds.left,
          height: layout.collapsed && !temporarilyExpanded ? 40 : bounds.bottom - bounds.top,
        },
      })) ?? [],
    relations: projectedRelations.flatMap(({ relation, sourceStructureNodeId, targetStructureNodeId }) =>
      projectRelation({
        relation,
        endpoints: relationEndpoints,
        sourceStructureNodeId,
        targetStructureNodeId,
        color: relationStyles.get(relation.id) ?? "neutral",
        parallelOffset: relationOffsets.get(relation.id) ?? 0,
      }),
    ),
  };
}

function projectRelation(options: {
  relation: SpaceRelation;
  endpoints: ReadonlyMap<string, RelationEndpoint>;
  sourceStructureNodeId: string;
  targetStructureNodeId: string;
  color: CanvasSceneRelation["color"];
  parallelOffset: number;
}): CanvasSceneRelation[] {
  const { relation, endpoints, sourceStructureNodeId, targetStructureNodeId, color, parallelOffset } =
    options;
  const source = endpoints.get(sourceStructureNodeId);
  const target = endpoints.get(targetStructureNodeId);
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
  byId: ReadonlyMap<string, CanvasSceneNode>,
) {
  return new Map(
    projected.flatMap(({ node }) => {
      const visible = byId.get(node.id);
      return visible ? [[node.id, { frame: visible.layout.frame, previewId: node.id }] as const] : [];
    }),
  );
}

function structureRelationEndpoints(scene: StructureScene) {
  const endpoints = new Map<string, RelationEndpoint>();
  for (const { bounds, structureNode } of scene.content)
    endpoints.set(structureNode.id, {
      frame: boundsFrame(bounds),
      previewId: structureNode.nodeId,
    });
  for (const { bounds, section, layout, temporarilyExpanded } of scene.sections)
    endpoints.set(section.id, {
      frame: boundsFrame(
        layout.collapsed && !temporarilyExpanded ? { ...bounds, bottom: bounds.top + 40 } : bounds,
      ),
      previewId: `structure:${section.id}`,
    });
  return endpoints;
}

function boundsFrame(bounds: { left: number; top: number; right: number; bottom: number }) {
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
  type ProjectedRelation = (typeof relations)[number];
  const groups = new Map<string, ProjectedRelation[]>();
  for (const projected of relations) {
    const key = [projected.sourceStructureNodeId, projected.targetStructureNodeId].toSorted().join("\u0000");
    const group = groups.get(key) ?? [];
    group.push(projected);
    groups.set(key, group);
  }
  const offsets = new Map<string, number>();
  for (const group of groups.values())
    group
      .toSorted((a, b) => a.relation.id.localeCompare(b.relation.id))
      .forEach((projected, index) => {
        const direction =
          projected.sourceStructureNodeId ===
          [projected.sourceStructureNodeId, projected.targetStructureNodeId].toSorted()[0]
            ? 1
            : -1;
        offsets.set(projected.relation.id, (index - (group.length - 1) / 2) * 20 * direction);
      });
  return offsets;
}
