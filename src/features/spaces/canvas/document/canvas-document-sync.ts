import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import {
  emptySpaceEditorDocument,
  parseSpaceEditorDocument,
  type SpaceEditorDocument,
} from "../contract/canvas-document";
import { spaceNodeSizePolicy, spaceNodeSupportsContentSizing } from "../node-size";

/** New content use-case only; existing persisted documents are never repaired from renderer nodes. */
export function appendEditorContent(
  document: SpaceEditorDocument,
  nodes: readonly SpaceCanvasNode[],
): SpaceEditorDocument {
  const first = document.structure.items.filter((item) => item.parentId === null).length;
  return parseSpaceEditorDocument({
    ...document,
    structure: {
      version: 1,
      items: [
        ...document.structure.items,
        ...nodes.map((node, index) => ({
          kind: "content",
          id: node.id,
          nodeId: node.id,
          parentId: null,
          order: first + index,
          locked: false,
        })),
      ],
    },
    layout: {
      version: 1,
      items: [
        ...document.layout.items,
        ...nodes.map((node) => {
          const { width, height } = spaceNodeSizePolicy(node.data);
          return {
            kind: "content",
            itemId: node.id,
            transform: { ...node.initialPosition, rotation: 0, scale: 1 },
            size: { width, height },
            sizeAuthority: spaceNodeSupportsContentSizing(node.data) ? "content" : "frame",
          };
        }),
      ],
    },
  });
}
export function createCanvasDocument(nodes: readonly SpaceCanvasNode[] = []): SpaceEditorDocument {
  return appendEditorContent(emptySpaceEditorDocument(), nodes);
}
