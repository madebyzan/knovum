import { compact, sortBy } from "es-toolkit";

import type { SpaceCanvasNode } from "../../../model/nodes/canvas-node";
import type { SpaceRelation } from "../../../model/relations/relation";
import { SECTION_HEADER_HEIGHT } from "../../../model/structure/section-metrics";
import type { WorldBounds } from "../../../model/structure/world-bounds";
import { canvasNodeInteraction } from "../../canvas-node-interaction";
import type { StructureScene } from "../../document/structure-scene";
import type { CanvasDocumentV1 } from "../../schema/document";
import type { CanvasScene, CanvasSceneNode } from "./canvas-scene";
import { projectSceneRelations, type RelationEndpoint } from "./scene-relations";

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
  const placedRelations =
    structure?.relations ??
    relations.map((relation) => ({
      relation,
      sourceStructureNodeId: relation.sourceId,
      targetStructureNodeId: relation.targetId,
    }));
  const sceneRelations = projectSceneRelations(placedRelations, relationEndpoints, colorByRelationId);
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
    relations: sceneRelations,
    relationKeyByRelationId: new Map(
      sceneRelations.flatMap(({ key, relations: drawn }) => drawn.map(({ id }) => [id, key] as const)),
    ),
  };
}

function contentRelationEndpoints(
  projected: readonly Omit<CanvasSceneNode, "paintOrder">[],
  sceneNodeById: ReadonlyMap<string, CanvasSceneNode>,
): ReadonlyMap<string, RelationEndpoint> {
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
