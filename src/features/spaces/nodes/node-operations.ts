import { invokeCommand } from "../../../foundation/ipc/invoke-command";
import { createSpaceNode } from "../canvas/node";
import { spaceNodeDefaultDimensions, spaceNodeSupportsContentSizing } from "../canvas/node-size";
import type { CreatedNodes } from "../contracts/generated/CreatedNodes";
import type { DiscardPendingContentRequest } from "../contracts/generated/DiscardPendingContentRequest";
import type { DiscardPendingContentResponse } from "../contracts/generated/DiscardPendingContentResponse";
import type { SpaceNodeProjection } from "../contracts/generated/SpaceNodeProjection";
import type { UpdateNodesResponse } from "../contracts/generated/UpdateNodesResponse";
import type { KnowledgeChunkSeed } from "../knowledge/model/schema/chunk";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceNodePayload } from "../model/nodes/node-payload";
import { validateSpaceNodePayload } from "../model/nodes/node-validation";
import { parseSpaceEditSnapshot, type SpaceEditSnapshot } from "../model/structure/schema/edit-snapshot";
import { invokeConfirmedMutation } from "../structure/confirmed-mutation";

export function parseSpaceNodes(rows: readonly SpaceNodeProjection[]): SpaceCanvasNode[] {
  return rows.map((row) =>
    createSpaceNode(validateSpaceNodePayload(row.data), { x: row.initialX, y: row.initialY }, row.id),
  );
}

interface NodeCreationMutation {
  mutationId: string;
  targetParentId: string | null;
}

interface CreateNodesInput {
  spaceId: string;
  nodes: readonly SpaceCanvasNode[];
  edit: NodeCreationMutation;
  sourceChunksByNodeId?: ReadonlyMap<string, readonly KnowledgeChunkSeed[]> | undefined;
}

/** The created nodes as the Space stored them, and the edit revision their creation reached. */
export interface CreatedSpaceNodes {
  editRevision: number;
  payloadById: ReadonlyMap<string, SpaceNodePayload>;
}

export async function createNodes({
  spaceId,
  nodes,
  edit,
  sourceChunksByNodeId = new Map(),
}: CreateNodesInput): Promise<CreatedSpaceNodes> {
  const created = await invokeConfirmedMutation<CreatedNodes>(
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
          sourceChunks: sourceChunksByNodeId.get(node.id) ?? [],
        })),
      },
    },
    edit.mutationId,
  );
  return {
    editRevision: created.editRevision,
    payloadById: new Map(created.nodes.map(({ id, data }) => [id, validateSpaceNodePayload(data)])),
  };
}

function spaceNodeLayoutInput(node: SpaceCanvasNode) {
  const size = spaceNodeDefaultDimensions(node.data.kind);
  return {
    width: size.width,
    height: size.height,
    sizingMode: spaceNodeSupportsContentSizing(node.data) ? ("content" as const) : ("frame" as const),
  };
}

export interface NodePayloadChangeInput {
  id: string;
  expectedContent: SpaceNodePayload;
  data: SpaceNodePayload;
  sourceChunks?: readonly KnowledgeChunkSeed[] | undefined;
}

export async function updateNodePayload(
  spaceId: string,
  change: NodePayloadChangeInput,
): Promise<{ data: SpaceNodePayload; editSnapshot: SpaceEditSnapshot | null }> {
  const { id, expectedContent, data, sourceChunks } = change;
  const committed = await updateNodePayloadMany(
    spaceId,
    [{ id, expectedContent, data }],
    sourceChunks ? new Map([[id, sourceChunks]]) : undefined,
  );
  const committedNode = committed.nodes.find((node) => node.id === id);
  if (!committedNode) throw new Error("Node update did not return its committed projection");
  return { data: committedNode.data, editSnapshot: committed.editSnapshot };
}

export async function updateNodePayloadMany(
  spaceId: string,
  changes: readonly { id: string; expectedContent: SpaceNodePayload; data: SpaceNodePayload }[],
  sourceChunksByNodeId?: ReadonlyMap<string, readonly KnowledgeChunkSeed[]>,
): Promise<{ nodes: Array<{ id: string; data: SpaceNodePayload }>; editSnapshot: SpaceEditSnapshot | null }> {
  if (!changes.length) return { nodes: [], editSnapshot: null };
  const committed = await invokeCommand<UpdateNodesResponse>("plugin:spaces|update_nodes", {
    request: {
      spaceId,
      changes: changes.map((change) => {
        const chunks = sourceChunksByNodeId?.get(change.id);
        return {
          ...change,
          ...(chunks ? { sourceChunks: chunks } : {}),
        };
      }),
    },
  });
  return {
    nodes: committed.nodes.map(({ id, data }) => ({ id, data: validateSpaceNodePayload(data) })),
    editSnapshot: committed.editSnapshot ? parseSpaceEditSnapshot(committed.editSnapshot) : null,
  };
}

export function discardPendingContent(request: DiscardPendingContentRequest) {
  return invokeConfirmedMutation<DiscardPendingContentResponse>(
    "plugin:spaces|discard_pending_content",
    { request },
    request.mutationId,
  );
}
