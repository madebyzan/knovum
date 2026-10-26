import { delay } from "es-toolkit";
import { beforeEach, expect, it, vi } from "vitest";
import { shallowRef } from "vue";

import { createSpaceHistoryState } from "../history/history-state";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceNodePayload } from "../model/nodes/node-payload";
import type { SpaceEditSnapshot } from "../model/structure/schema/edit-snapshot";
import { createSpaceMutations } from "../session/actions/node-mutations";

const invoke = vi.hoisted(() =>
  vi.fn<(command: string, payload?: Record<string, unknown>) => Promise<unknown>>(),
);
vi.mock("../../../foundation/ipc/invoke-command", () => ({ invokeCommand: invoke }));

const CREATE_NODES = "plugin:spaces|create_nodes";
const EDIT_SNAPSHOT = "plugin:spaces|load_space_edit_snapshot";
/** `InsertNodesRequest` denies unknown fields; content creation carries no caller revision. */
const CREATE_NODES_FIELDS = "mutationId,nodes,spaceId,targetParentId";

interface RequestedNode {
  id: string;
  data: SpaceNodePayload;
}

/**
 * Appends inside its own write transaction and answers with the nodes as stored, revisioned by
 * Rust, like the command does. Anything else a creation asked for — the whole Space above all —
 * is refused, so the session can only show a node from that answer and the Space's geometry.
 */
function committedSpace(readDelayMs: (read: number) => number = () => 0) {
  let revision = 0;
  let reads = 0;
  invoke.mockImplementation(async (command, payload) => {
    if (command === CREATE_NODES) {
      const request = (payload as { request: Record<string, unknown> }).request;
      const fields = Object.keys(request).toSorted().join(",");
      if (fields !== CREATE_NODES_FIELDS)
        throw new Error(`create_nodes received ${fields}`, { cause: "unknown-field" });
      await delay(1);
      revision += 1;
      const nodes = (request.nodes as RequestedNode[]).map(({ id, data }) => ({
        id,
        data: { ...data, knowledgeRevision: `stored-${id}` },
      }));
      return { editRevision: revision, nodes };
    }
    if (command === EDIT_SNAPSHOT) {
      const snapshot = { editRevision: revision, structure: [], layouts: [], relationStyles: [] };
      reads += 1;
      await delay(readDelayMs(reads));
      return snapshot;
    }
    throw new Error(`Unexpected command ${command}`, { cause: command });
  });
}

function session() {
  const nodes = shallowRef<SpaceCanvasNode[]>([]);
  const editSnapshot = shallowRef<SpaceEditSnapshot>({
    editRevision: 0,
    structure: [],
    layouts: [],
    relationStyles: [],
  });
  const mutations = createSpaceMutations({
    nodes,
    editSnapshot,
    groupScopePath: shallowRef<string[]>([]),
    activeSpaceId: shallowRef<string | undefined>("space"),
    history: createSpaceHistoryState().history,
    nextInsertionPoint: () => ({ x: 0, y: 0 }),
    ensureActiveSpacePersisted: () => Promise.resolve(),
    recordCommittedSpaceEdit: () => undefined,
  });
  return { nodes, editSnapshot, mutations };
}

function droppedImage(index: number): SpaceNodePayload {
  return {
    kind: "image",
    title: `image-${index}.png`,
    assetPath: `/managed/image-${index}.png`,
    fileName: `image-${index}.png`,
  };
}

function importDrop(space: ReturnType<typeof session>, count: number) {
  return Promise.all(
    Array.from({ length: count }, (_, index) =>
      space.mutations.createPendingContentForSpace("space", droppedImage(index), {
        x: index * 320,
        y: 0,
      }),
    ),
  );
}

beforeEach(() => {
  invoke.mockReset();
});

it("commits every file of one multi-file drop", async () => {
  committedSpace();
  const space = session();

  const imported = await importDrop(space, 6);

  expect(imported.flat()).toHaveLength(6);
  expect(space.nodes.value.map(({ data }) => data.title)).toEqual(
    Array.from({ length: 6 }, (_, index) => `image-${index}.png`),
  );
  expect(space.editSnapshot.value.editRevision).toBe(6);
});

it("shows a created node as the Space stored it", async () => {
  committedSpace();
  const space = session();

  const [created] = await space.mutations.createPendingContentForSpace("space", droppedImage(0));

  expect(space.nodes.value.map(({ id, data }) => [id, data.knowledgeRevision])).toEqual([
    [created?.id, `stored-${created?.id}`],
  ]);
  expect(created?.data.knowledgeRevision).toBe(`stored-${created?.id}`);
  expect(space.editSnapshot.value.editRevision).toBe(1);
});

it("keeps the newest Space when parallel snapshot reads settle out of order", async () => {
  committedSpace((read) => (8 - read) * 15);
  const space = session();

  await importDrop(space, 6);

  expect(space.nodes.value).toHaveLength(6);
  expect(space.editSnapshot.value.editRevision).toBe(6);
});
