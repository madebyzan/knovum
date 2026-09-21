import { invoke } from "@tauri-apps/api/core";

import { createSpaceNode } from "../canvas/node";
import { spaceNodeSizePolicy, spaceNodeSupportsContentSizing } from "../canvas/node-size";
import type { KnowledgeChunkSeed } from "../knowledge/model/schema";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceNodePayload } from "../model/nodes/node-payload";
import { validateSpaceNodePayload } from "../model/nodes/schema/node-payload";
import { invokeConfirmedMutation } from "../organization/confirmed-mutation";

interface NodeRecord {
  id: string;
  initialX: number;
  initialY: number;
  data: unknown;
}

export async function listNodes(spaceId: string): Promise<SpaceCanvasNode[]> {
  const rows: NodeRecord[] = await invoke<NodeRecord[]>("plugin:spaces|list_nodes", { spaceId });
  return rows.map((row) =>
    createSpaceNode(validateSpaceNodePayload(row.data), { x: row.initialX, y: row.initialY }, row.id),
  );
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
  id: string,
  data: SpaceNodePayload,
  sourceChunks?: readonly KnowledgeChunkSeed[],
): Promise<void> {
  await updateNodePayloadMany(
    spaceId,
    [{ id, data }],
    sourceChunks ? new Map([[id, sourceChunks]]) : undefined,
  );
}

export async function updateNodePayloadMany(
  spaceId: string,
  changes: readonly { id: string; data: SpaceNodePayload }[],
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

export interface DiscardPendingContentResponse {
  mutationId: string;
  appliedAtRevision: number;
  assetCleanupCandidates: string[];
}

export function discardPendingContent(request: {
  spaceId: string;
  nodeId: string;
  expectedEditRevision: number;
  mutationId: string;
}) {
  return invokeConfirmedMutation<DiscardPendingContentResponse>(
    "plugin:spaces|discard_pending_content",
    { request },
    request.mutationId,
  );
}
