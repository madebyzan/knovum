import { flushPromises } from "@vue/test-utils";
import { beforeEach, expect, it, vi } from "vitest";

import { deferred } from "../../../../shared/__tests__/support/deferred";
import { createTestSessionContext, createTestSpace } from "../../__tests__/support/session-context";
import { createSpaceNode } from "../../canvas/node";
import { createSpaceNotePayload } from "../../document-editing/tiptap-content";
import { beginSpaceMutation } from "../../history/mutation-barrier";
import type { SpaceSessionContent } from "../../session/space-projection";
import { closeWorkbenchTab } from "../workbench-close-actions";
import { openSpace } from "../workbench-navigation-actions";
import { findSpaceCanvasTab } from "../workbench-tab";

const repository = vi.hoisted(() => ({
  load: vi.fn<(spaceId: string) => Promise<SpaceSessionContent>>(),
}));
vi.mock("../../session/space-projection", () => ({ loadSpaceSessionProjection: repository.load }));
vi.mock("../space-operations", () => ({
  markSpaceOpened: vi.fn<() => Promise<void>>().mockResolvedValue(),
}));

function content(...nodeIds: string[]): SpaceSessionContent {
  return {
    nodes: nodeIds.map((id) => createSpaceNode(createSpaceNotePayload(id), { x: 0, y: 0 }, id)),
    scenes: [],
    relations: [],
    editSnapshot: { editRevision: 0, structure: [], layouts: [], relationStyles: [] },
  };
}

function shownNodeIds(context: ReturnType<typeof createTestSessionContext>) {
  return context.nodes.value.map(({ id }) => id);
}

/** Inbox is active; Research was opened from storage, so leaving it retained its session. */
async function withResearchRetained() {
  const context = createTestSessionContext({
    spaces: [createTestSpace(), createTestSpace({ id: "research", kind: "space", title: "Research" })],
  });
  context.nodes.value = content("inbox-note").nodes;
  repository.load.mockResolvedValueOnce(content("research-note"));
  await openSpace(context, "research");
  repository.load.mockResolvedValueOnce(content("inbox-note"));
  await openSpace(context, "inbox");
  await flushPromises();
  repository.load.mockClear();
  return context;
}

beforeEach(() => repository.load.mockReset());

it("shows a Space it retained at once and revalidates it in the background", async () => {
  const context = await withResearchRetained();
  const read = deferred<SpaceSessionContent>();
  repository.load.mockReturnValueOnce(read.promise);

  await openSpace(context, "research");

  expect(context.activeSpaceId.value).toBe("research");
  expect(shownNodeIds(context)).toEqual(["research-note"]);
  expect(repository.load).toHaveBeenCalledExactlyOnceWith("research");
  read.resolve(content("research-note", "published-meanwhile"));
  await flushPromises();
  expect(shownNodeIds(context)).toEqual(["research-note", "published-meanwhile"]);
});

it("reads again when a local edit overlapped the revalidation", async () => {
  const context = await withResearchRetained();
  const overlapped = deferred<SpaceSessionContent>();
  repository.load.mockReturnValueOnce(overlapped.promise).mockResolvedValueOnce(content("after-edit"));

  await openSpace(context, "research");
  await vi.waitFor(() => expect(repository.load).toHaveBeenCalledOnce());
  beginSpaceMutation(context.history)();
  overlapped.resolve(content("before-edit"));
  await flushPromises();

  expect(repository.load).toHaveBeenCalledTimes(2);
  expect(shownNodeIds(context)).toEqual(["after-edit"]);
});

it("releases a retained session with its Space's last tab", async () => {
  const context = await withResearchRetained();
  await closeWorkbenchTab(context, findSpaceCanvasTab(context.tabs.value, "research")!.id);
  expect(context.runtime.retained.has("research")).toBe(false);
  repository.load.mockResolvedValueOnce(content("research-note"));

  await openSpace(context, "research");

  expect(repository.load).toHaveBeenCalledExactlyOnceWith("research");
});

it("keeps the object of every node a revalidation found unchanged", async () => {
  const context = await withResearchRetained();
  const [kept] = context.runtime.retained.get("research")!.content.nodes;
  repository.load.mockResolvedValueOnce(content("research-note", "published-meanwhile"));
  await openSpace(context, "research");
  await flushPromises();

  expect(context.nodes.value[0]).toBe(kept);
  expect(shownNodeIds(context)).toEqual(["research-note", "published-meanwhile"]);
});
