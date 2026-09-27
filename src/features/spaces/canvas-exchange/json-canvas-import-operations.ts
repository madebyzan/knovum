import type { ImportJsonCanvasResult } from "../contracts/generated/ImportJsonCanvasResult";
import type { KnowledgeChunkSeed } from "../knowledge/model/schema";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceRelation } from "../model/relations/relation";
import type { SpaceEditSnapshot } from "../model/structure/edit-snapshot";
import { invokeConfirmedMutation } from "../structure/confirmed-mutation";

export interface PreparedJsonCanvasNode {
  sourceId: string;
  node: SpaceCanvasNode;
  sourceChunks?: readonly KnowledgeChunkSeed[];
}

export function importJsonCanvas(input: {
  spaceId: string;
  mutationId: string;
  nodes: readonly PreparedJsonCanvasNode[];
  relations: readonly SpaceRelation[];
  baseRevision: number;
  snapshot: SpaceEditSnapshot;
}) {
  const { baseRevision, mutationId, nodes, relations, snapshot, spaceId } = input;
  return invokeConfirmedMutation<ImportJsonCanvasResult>(
    "plugin:spaces|import_json_canvas_nodes",
    {
      request: {
        spaceId,
        mutationId,
        baseRevision,
        structureNodes: snapshot.structure,
        layouts: snapshot.layouts,
        relationStyles: snapshot.relationStyles,
        relations,
        nodes: nodes.map(({ sourceId, node, sourceChunks }) => {
          const layout = layoutFor(snapshot, node.id);
          return {
            sourceId,
            node: {
              id: node.id,
              kind: node.data.kind,
              initialX: node.initialPosition.x,
              initialY: node.initialPosition.y,
              width: layout.size.width,
              height: layout.size.height,
              sizeAuthority: layout.sizeAuthority,
              appearance: layout.appearance,
              fileSubpath: layout.fileSubpath,
              data: node.data,
              sourceChunks: sourceChunks ?? [],
            },
          };
        }),
      },
    },
    mutationId,
  );
}

function layoutFor(snapshot: SpaceEditSnapshot, structureNodeId: string) {
  const layout = snapshot.layouts.find(
    (candidate): candidate is Extract<(typeof snapshot.layouts)[number], { kind: "content" }> =>
      candidate.structureNodeId === structureNodeId && candidate.kind === "content",
  );
  if (!layout) throw new Error(`JSON Canvas content layout is unavailable: ${structureNodeId}`);
  return layout;
}
