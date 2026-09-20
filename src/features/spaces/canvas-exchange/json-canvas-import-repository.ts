import { invoke } from "@tauri-apps/api/core";

import type { SpaceEditorDocument } from "../canvas/contract/canvas-document";
import type { KnowledgeChunkSeed } from "../knowledge/model/schema";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceRelation } from "../model/relations/relation";

export interface PreparedJsonCanvasNode {
  sourceId: string;
  node: SpaceCanvasNode;
  sourceChunks?: readonly KnowledgeChunkSeed[];
}

export function importJsonCanvas(input: {
  spaceId: string;
  nodes: readonly PreparedJsonCanvasNode[];
  relations: readonly SpaceRelation[];
  baseRevision: number;
  document: SpaceEditorDocument;
}) {
  const { baseRevision, document, nodes, relations, spaceId } = input;
  return invoke<{ importedNodes: readonly { sourceId: string; nodeId: string }[]; revision: number }>(
    "plugin:spaces|import_json_canvas_nodes",
    {
      request: {
        spaceId,
        baseRevision,
        document,
        relations,
        nodes: nodes.map(({ sourceId, node, sourceChunks }) => ({
          sourceId,
          node: {
            id: node.id,
            kind: node.data.kind,
            initialX: node.initialPosition.x,
            initialY: node.initialPosition.y,
            data: node.data,
            sourceChunks: sourceChunks ?? [],
          },
        })),
      },
    },
  );
}
