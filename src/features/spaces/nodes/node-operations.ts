import { invokeCommand as invoke } from "../../../foundation/ipc/invoke-command";
import { createSpaceNode } from "../canvas/node";
import { spaceNodeSizePolicy, spaceNodeSupportsContentSizing } from "../canvas/node-size";
import type { DiscardPendingContentRequest } from "../contracts/generated/DiscardPendingContentRequest";
import type { DiscardPendingContentResponse } from "../contracts/generated/DiscardPendingContentResponse";
import type { SpaceNodeRecord } from "../contracts/generated/SpaceNodeRecord";
import type { KnowledgeChunkSeed } from "../knowledge/model/schema";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceNodePayload } from "../model/nodes/node-payload";
import { validateSpaceNodePayload } from "../model/nodes/schema/node-payload";
import { invokeConfirmedMutation } from "../structure/confirmed-mutation";

export function parseSpaceNodeRecords(rows: readonly SpaceNodeRecord[]): SpaceCanvasNode[] {
  return rows.map((row) =>
    createSpaceNode(validateSpaceNodePayload(row.data), { x: row.initialX, y: row.initialY }, row.id),
  );
}

export async function getNode(spaceId: string, nodeId: string): Promise<SpaceCanvasNode> {
  const row = await invoke<SpaceNodeRecord>("plugin:spaces|get_node", { spaceId, nodeId });
  return createSpaceNode(validateSpaceNodePayload(row.data), { x: row.initialX, y: row.initialY }, row.id);
}

export async function createNodes(
  spaceId: string,
  nodes: readonly SpaceCanvasNode[],
  edit: {
    mutationId: string;
    expectedEditRevision: number;
    targetParentId: string | null;
  },
  sourceChunks: ReadonlyMap<string, readonly KnowledgeChunkSeed[]> = new Map(),
): Promise<number> {
  return invokeConfirmedMutation<number>(
    "plugin:spaces|create_nodes",
    {
      request: {
        spaceId,
        ...edit,
        nodes: nodes.map((node) => ({
          ...spaceNodeLayoutInput(node),
          id: node.id,
          kind: node.data.kind,
          initialX: node.initialPosition.x,
          initialY: node.initialPosition.y,
          data: node.data,
          sourceChunks: sourceChunks.get(node.id) ?? [],
        })),
      },
    },
    edit.mutationId,
  );
}

function spaceNodeLayoutInput(node: SpaceCanvasNode) {
  const size = spaceNodeSizePolicy(node.data);
  return {
    width: size.width,
    height: size.height,
    sizeAuthority: spaceNodeSupportsContentSizing(node.data) ? ("content" as const) : ("frame" as const),
  };
}

export async function updateNodePayload(
  spaceId: string,
  change: {
    id: string;
    expectedData: SpaceNodePayload;
    data: SpaceNodePayload;
    sourceChunks?: readonly KnowledgeChunkSeed[] | undefined;
  },
): Promise<void> {
  const { id, expectedData, data, sourceChunks } = change;
  await updateNodePayloadMany(
    spaceId,
    [{ id, expectedData, data }],
    sourceChunks ? new Map([[id, sourceChunks]]) : undefined,
  );
}

export async function updateNodePayloadMany(
  spaceId: string,
  changes: readonly { id: string; expectedData: SpaceNodePayload; data: SpaceNodePayload }[],
  sourceChunks?: ReadonlyMap<string, readonly KnowledgeChunkSeed[]>,
): Promise<void> {
  if (!changes.length) return;
  await invoke("plugin:spaces|update_nodes", {
    request: {
      spaceId,
      changes: changes.map((change) => {
        const chunks = sourceChunks?.get(change.id);
        return {
          ...change,
          ...(chunks ? { sourceChunks: chunks } : {}),
        };
      }),
    },
  });
}

export function discardPendingContent(request: DiscardPendingContentRequest) {
  return invokeConfirmedMutation<DiscardPendingContentResponse>(
    "plugin:spaces|discard_pending_content",
    { request },
    request.mutationId,
  );
}
