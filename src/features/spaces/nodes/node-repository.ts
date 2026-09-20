import { invoke } from "@tauri-apps/api/core";

import { appendEditorContent } from "../canvas/document/canvas-document-sync";
import { createSpaceNode } from "../canvas/node";
import { transactSpaceEditor } from "../canvas/persistence/canvas-persistence-coordinator";
import type { SpaceDeletedNode } from "../deletion/deleted-node";
import type { KnowledgeChunkSeed } from "../knowledge/model/schema";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceNodePayload } from "../model/nodes/node-payload";
import { validateSpaceNodePayload } from "../model/nodes/schema/node-payload";

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
  sourceChunks: ReadonlyMap<string, readonly KnowledgeChunkSeed[]> = new Map(),
): Promise<void> {
  if (!nodes.length) return;
  await transactSpaceEditor(spaceId, async (before, baseRevision) => {
    const document = appendEditorContent(before, nodes);
    const saved = await invoke<{ revision: number }>("plugin:spaces|create_nodes", {
      request: {
        spaceId,
        baseRevision,
        document,
        nodes: nodes.map((node) => ({
          id: node.id,
          kind: node.data.kind,
          initialX: node.initialPosition.x,
          initialY: node.initialPosition.y,
          data: node.data,
          sourceChunks: sourceChunks.get(node.id) ?? [],
        })),
      },
    });
    return { document, revision: saved.revision };
  });
}

export async function trashNodes(spaceId: string, ids: readonly string[]): Promise<void> {
  if (!ids.length) return;
  await invoke("plugin:spaces|trash_nodes", { request: { spaceId, ids } });
}

export async function restoreNodes(spaceId: string, ids: readonly string[]): Promise<void> {
  if (!ids.length) return;
  await invoke("plugin:spaces|restore_nodes", { request: { spaceId, ids } });
}

export async function setNodesDeleted(
  spaceId: string,
  ids: readonly string[],
  deleted: boolean,
): Promise<void> {
  if (deleted) await trashNodes(spaceId, ids);
  else await restoreNodes(spaceId, ids);
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

export function listRecentlyDeletedNodes(spaceId: string | null): Promise<SpaceDeletedNode[]> {
  return invoke("plugin:spaces|list_recently_deleted_items", { spaceId });
}

export async function permanentlyDeleteNodes(
  selections: readonly { spaceId: string; ids: readonly string[] }[],
): Promise<string[]> {
  if (!selections.length) return [];
  return invoke<string[]>("plugin:spaces|delete_deleted_nodes", { request: { selections } });
}

export async function moveNodesBetweenSpaces(
  sourceSpaceId: string,
  targetSpaceId: string,
  ids: readonly string[],
): Promise<string[]> {
  if (sourceSpaceId === targetSpaceId || !ids.length) return [];
  return invoke<string[]>("plugin:spaces|move_space_selection", {
    request: { sourceSpaceId, targetSpaceId, ids },
  });
}
