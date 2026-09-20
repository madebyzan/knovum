import type { CanvasSelectionTransfer } from "../canvas/contract/canvas-editor";
import type { SpaceKnowledgeObjectType } from "../knowledge/model/object-types";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceRelation } from "../model/relations/relation";
import { structureSubtree } from "../model/structure/structure-index";
import { indexStructureSnapshot } from "../organization/structure/structure-context";
import { projectStructure } from "../organization/structure/structure-projection";
import { structureOperationRoots } from "../organization/structure/structure-selection";
import { parseSpaceClipboardPayload } from "./payload-schema";

/** A detached native fragment includes hidden descendants and full local geometry. */
export function createSpaceClipboardPayload(input: {
  spaceId: string;
  transfer: CanvasSelectionTransfer;
  nodes: readonly SpaceCanvasNode[];
  relations: readonly SpaceRelation[];
  objectTypes: readonly SpaceKnowledgeObjectType[];
}) {
  const { document, itemIds } = input.transfer;
  const context = indexStructureSnapshot(document);
  const geometry = projectStructure(context);
  const roots = structureOperationRoots(context.tree, itemIds).toSorted(
    (a, b) => geometry.paintOrder.get(a.id)! - geometry.paintOrder.get(b.id)!,
  );
  const order = new Map(roots.map((item, index) => [item.id, index]));
  const selected = structureSubtree(
    context.tree,
    roots.map((item) => item.id),
  );
  const ids = new Set(selected.flatMap((item) => (item.kind === "content" ? [item.nodeId] : [])));
  const nodes = input.nodes.filter((node) => ids.has(node.id)).map(({ id, data }) => ({ id, data }));
  const types = new Set(nodes.flatMap(({ data }) => (data.kind === "object" ? [data.objectTypeId] : [])));
  const colors = new Map(document.relationStyles?.map((style) => [style.relationId, style.color]));
  return parseSpaceClipboardPayload({
    version: 1,
    kind: "space-selection",
    sourceSpaceId: input.spaceId,
    structure: {
      version: 1,
      items: selected.map((item) =>
        order.has(item.id) ? { ...item, parentId: null, order: order.get(item.id)! } : item,
      ),
    },
    layout: {
      version: 1,
      items: selected.map((item) => {
        const layout = context.layouts.get(item.id)!;
        if (!order.has(item.id) || layout.kind === "section") return layout;
        return { ...layout, transform: geometry.transforms.get(item.id)! };
      }),
    },
    nodes,
    objectTypes: input.objectTypes
      .filter((type) => !type.system && types.has(type.id))
      .map(({ id, name, icon, description, fields }) => ({ id, name, icon, description, fields })),
    relations: input.relations
      .filter(
        (relation) =>
          relation.status !== "invalidated" && ids.has(relation.sourceId) && ids.has(relation.targetId),
      )
      .map((relation) => ({
        id: relation.id,
        sourceId: relation.sourceId,
        targetId: relation.targetId,
        kind: relation.kind,
        rationale: relation.rationale,
        status: relation.status,
        ...(relation.confidence !== undefined ? { confidence: relation.confidence } : {}),
        ...(colors.has(relation.id) ? { color: colors.get(relation.id) } : {}),
      })),
  });
}
