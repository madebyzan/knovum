import type { SpaceCanvasNode } from "../../../model/nodes/canvas-node";
import type { SpaceRelation } from "../../../model/relations/relation";
import type { SpaceEditorDocument, CanvasNodeLayoutV1 } from "../../contract/canvas-document";
import { spaceNodeDimensions } from "../../node-size";
import { canvasNodeInteraction } from "./canvas-node-interaction";
import type { CanvasScene, CanvasSceneNode, CanvasSceneRelation, CanvasSceneSection } from "./canvas-scene";
import { parallelRelationOffsets } from "./parallel-relation-offsets";
import { relationGeometry } from "./relation-path";

export type { CanvasScene, CanvasSceneNode, CanvasSceneRelation, CanvasSceneSection } from "./canvas-scene";

export function projectCanvasScene(
  document: SpaceEditorDocument,
  nodes: readonly SpaceCanvasNode[],
  relations: readonly SpaceRelation[],
): CanvasScene {
  const content = new Map(nodes.map((node) => [node.id, node]));
  const projected = document.nodes
    .flatMap<Omit<CanvasSceneNode, "paintOrder">>((layout) => {
      const node = content.get(layout.nodeId);
      return node && !node.hidden
        ? [
            {
              interaction: canvasNodeInteraction(node),
              layout: effectiveLayout(layout, node),
              node,
            },
          ]
        : [];
    })
    .toSorted((left, right) => left.layout.zIndex - right.layout.zIndex);
  const collapsedSections = new Set(
    projected
      .filter(({ node }) => node.data.kind === "section" && node.data.collapsed)
      .map(({ node }) => node.id),
  );
  const visibleNodes = projected.filter(
    ({ node }) => !node.data.sectionId || !collapsedSections.has(node.data.sectionId),
  );
  const backgroundNodes = visibleNodes.filter(({ node }) => node.data.kind === "section");
  const contentNodes = visibleNodes.filter(({ node }) => node.data.kind !== "section");
  const paintOrderedNodes = [...backgroundNodes, ...contentNodes].map((node, paintOrder) =>
    Object.assign({}, node, { paintOrder }),
  );
  const byId = new Map(paintOrderedNodes.map((item) => [item.node.id, item]));
  const relationEndpoints = relationEndpointNodes(projected, byId, collapsedSections);
  const relationStyles = new Map(document.relationStyles?.map((style) => [style.relationId, style.color]));
  const relationOffsets = parallelRelationOffsets(relations, relationEndpoints);
  const membersBySection = new Map<string, CanvasSceneNode[]>();
  for (const item of paintOrderedNodes) {
    if (!item.node.data.sectionId) continue;
    const members = membersBySection.get(item.node.data.sectionId) ?? [];
    members.push(item);
    membersBySection.set(item.node.data.sectionId, members);
  }
  const sections = backgroundNodes
    .filter((item) => item.node.data.kind === "section")
    .map((section): CanvasSceneSection =>
      Object.assign({}, byId.get(section.node.id)!, {
        members: membersBySection.get(section.node.id) ?? [],
      }),
    );
  return {
    contentNodes: paintOrderedNodes.filter(({ node }) => node.data.kind !== "section"),
    nodes: paintOrderedNodes,
    nodesById: byId,
    sections,
    relations: relations.flatMap((relation) =>
      projectRelation(
        relation,
        relationEndpoints,
        relationStyles.get(relation.id) ?? "neutral",
        relationOffsets.get(relation.id) ?? 0,
      ),
    ),
  };
}

function relationEndpointNodes(
  nodes: readonly Omit<CanvasSceneNode, "paintOrder">[],
  visibleById: ReadonlyMap<string, CanvasSceneNode>,
  collapsedSections: ReadonlySet<string>,
) {
  const endpoints = new Map<string, CanvasSceneNode>();
  for (const item of nodes) {
    const sectionId = item.node.data.sectionId;
    const representativeId = sectionId && collapsedSections.has(sectionId) ? sectionId : item.node.id;
    const representative = visibleById.get(representativeId);
    if (representative) endpoints.set(item.node.id, representative);
  }
  return endpoints;
}

function effectiveLayout(layout: CanvasNodeLayoutV1, node: SpaceCanvasNode): CanvasNodeLayoutV1 {
  if (node.data.kind !== "section" || !node.data.collapsed) return layout;
  return { ...layout, frame: { ...layout.frame, ...spaceNodeDimensions(node.data) } };
}

function projectRelation(
  relation: SpaceRelation,
  nodes: ReadonlyMap<string, CanvasSceneNode>,
  color: CanvasSceneRelation["color"],
  parallelOffset: number,
): CanvasSceneRelation[] {
  const source = nodes.get(relation.sourceId)?.layout.frame;
  const target = nodes.get(relation.targetId)?.layout.frame;
  const sourceNodeId = nodes.get(relation.sourceId)?.node.id;
  const targetNodeId = nodes.get(relation.targetId)?.node.id;
  if (!source || !target || !sourceNodeId || !targetNodeId || sourceNodeId === targetNodeId) return [];
  const geometry = relationGeometry(source, target, parallelOffset);
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
      sourceNodeId,
      targetNodeId,
      parallelOffset,
    },
  ];
}
