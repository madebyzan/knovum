import type { SpaceCanvasNode } from "../../../model/nodes/canvas-node";
import type { SpaceRelation } from "../../../model/relations/relation";
import { structureAncestors } from "../../../model/structure/structure-index";
import { indexStructureSnapshot } from "../../../organization/structure/structure-context";
import {
  structureContentIds,
  structureSelectionTarget,
} from "../../../organization/structure/structure-selection";
import { visibleStructureRepresentative } from "../../../organization/structure/structure-visibility";
import type { CanvasDisplayNode } from "../../contract/canvas-display-node";
import type { SpaceEditorDocument } from "../../contract/canvas-document";
import { projectEditorLayouts } from "../../document/editor-layout-projection";
import { canvasNodeInteraction } from "./canvas-node-interaction";
import type {
  CanvasScene,
  CanvasSceneNode,
  CanvasSceneContentNode,
  CanvasSceneRelation,
} from "./canvas-scene";
import { parallelRelationOffsets } from "./parallel-relation-offsets";
import { relationGeometry } from "./relation-path";
export type {
  CanvasScene,
  CanvasSceneNode,
  CanvasSceneContentNode,
  CanvasSceneRelation,
  CanvasSceneSection,
} from "./canvas-scene";

export function projectCanvasScene(
  document: SpaceEditorDocument,
  nodes: readonly SpaceCanvasNode[],
  relations: readonly SpaceRelation[],
  scopeGroupId: string | null = null,
): CanvasScene {
  const context = indexStructureSnapshot(document);
  const content = new Map(nodes.map((node) => [node.id, node]));
  const layouts = projectEditorLayouts(document);
  const layoutsById = new Map(layouts.map((layout) => [layout.nodeId, layout]));
  const projected = layouts
    .flatMap<CanvasSceneNode>((layout) => {
      const item = context.tree.items.get(layout.nodeId)!;
      if (visibleStructureRepresentative(context, item.id) !== item.id) return [];
      let node: CanvasDisplayNode;
      if (item.kind === "content") {
        const source = content.get(item.nodeId);
        if (!source || source.hidden) return [];
        node = source;
      } else {
        const local = context.layouts.get(item.id)!;
        const contentCount = structureContentIds(context.tree, [item.id]).length;
        node =
          item.kind === "group"
            ? { id: item.id, data: { kind: "group", title: item.title, collapsed: false, contentCount } }
            : {
                id: item.id,
                data: {
                  kind: "section",
                  title: item.title,
                  collapsed: local.kind === "section" && local.collapsed,
                  contentCount,
                },
              };
      }
      const targetId = structureSelectionTarget(context.tree, item.id, scopeGroupId);
      const target = targetId ? context.tree.items.get(targetId) : undefined;
      const locked =
        !target ||
        target.locked ||
        structureAncestors(context.tree, target.id).some((parent) => parent.locked);
      const interaction = canvasNodeInteraction(node, locked);
      if (targetId !== item.id) interaction.connectable = false;
      return [{ node, layout, paintOrder: layout.zIndex, interaction }];
    })
    .toSorted((left, right) => left.paintOrder - right.paintOrder);
  const ordered = [
    ...projected.filter((item) => item.node.data.kind === "section"),
    ...projected.filter((item) => item.node.data.kind !== "section"),
  ];
  const byId = new Map(ordered.map((item) => [item.node.id, item]));
  const endpoints = new Map<string, CanvasSceneNode>();
  for (const item of document.structure.items) {
    if (item.kind !== "content") continue;
    const representative = byId.get(visibleStructureRepresentative(context, item.id));
    if (representative && !content.get(item.nodeId)?.hidden) endpoints.set(item.nodeId, representative);
  }
  const offsets = parallelRelationOffsets(relations, endpoints);
  const styles = new Map(document.relationStyles?.map((style) => [style.relationId, style.color]));
  return {
    nodes: ordered,
    nodesById: byId,
    contentNodes: ordered.filter(
      (item): item is CanvasSceneContentNode =>
        item.node.data.kind !== "group" && item.node.data.kind !== "section",
    ),
    groups: ordered.filter((item) => item.node.data.kind === "group"),
    sections: ordered
      .filter((item) => item.node.data.kind === "section")
      .map((section) => ({
        ...section,
        members: structureContentIds(context.tree, [section.node.id]).flatMap((id) => {
          const node = content.get(id);
          const layout = layoutsById.get(id);
          return node && layout
            ? [{ node, layout, interaction: canvasNodeInteraction(node), paintOrder: layout.zIndex }]
            : [];
        }),
      })),
    relations: relations.flatMap((relation) =>
      projectRelation(
        relation,
        endpoints,
        styles.get(relation.id) ?? "neutral",
        offsets.get(relation.id) ?? 0,
      ),
    ),
  };
}
function projectRelation(
  relation: SpaceRelation,
  nodes: ReadonlyMap<string, CanvasSceneNode>,
  color: CanvasSceneRelation["color"],
  parallelOffset: number,
): CanvasSceneRelation[] {
  const source = nodes.get(relation.sourceId),
    target = nodes.get(relation.targetId);
  if (!source || !target || source.node.id === target.node.id || relation.status === "invalidated") return [];
  const geometry = relationGeometry(source.layout.frame, target.layout.frame, parallelOffset);
  return [
    {
      ...geometry,
      id: relation.id,
      key: `relation:${relation.id}`,
      kind: "relation",
      label: relation.kind,
      relation,
      color,
      sourceNodeId: source.node.id,
      targetNodeId: target.node.id,
      parallelOffset,
    },
  ];
}
