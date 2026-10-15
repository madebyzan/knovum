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
const SESSION_PROJECTION = "plugin:spaces|load_space_session_projection";
/** `InsertNodesRequest` denies unknown fields; content creation carries no caller revision. */
const CREATE_NODES_FIELDS = "mutationId,nodes,spaceId,targetParentId";

interface CommittedNode {
  id: string;
  initialX: number;
  initialY: number;
  data: SpaceNodePayload;
}

/** Appends inside its own write transaction, like the Rust command does. */
function committedSpace(readDelayMs: (read: number) => number = () => 0) {
  let revision = 0;
  let reads = 0;
  const committed: CommittedNode[] = [];
  invoke.mockImplementation(async (command, payload) => {
    if (command === CREATE_NODES) {
      const request = (payload as { request: Record<string, unknown> }).request;
      const fields = Object.keys(request).toSorted().join(",");
      if (fields !== CREATE_NODES_FIELDS)
        throw new Error(`create_nodes received ${fields}`, { cause: "unknown-field" });
      await delay(1);
      committed.push(...(request.nodes as CommittedNode[]));
      revision += 1;
      return revision;
    }
    if (command === SESSION_PROJECTION) {
      const projection = {
        nodes: committed.map(({ id, initialX, initialY, data }) => ({ id, initialX, initialY, data })),
        scenes: [],
        relations: [],
        editSnapshot: { editRevision: revision, structure: [], layouts: [], relationStyles: [] },
      };
      reads += 1;
      await delay(readDelayMs(reads));
      return projection;
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

it("keeps the newest Space when parallel projection reads settle out of order", async () => {
  committedSpace((read) => (8 - read) * 15);
  const space = session();

  await importDrop(space, 6);

  expect(space.nodes.value).toHaveLength(6);
  expect(space.editSnapshot.value.editRevision).toBe(6);
});
