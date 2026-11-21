import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { createTestSpace } from "../../__tests__/support/session-context";
import type { Space } from "../../model/space";
import { createSpaceState } from "../../session/session-state";
import type { loadSpaceSessionProjection } from "../../session/space-projection";
import { createSpaceSessionContext } from "../../session/space-session-composition";
import type { SpaceSessionContext } from "../space-session-context";
import { restoreWorkbench } from "../workbench-restore-actions";
import type { WorkbenchSession } from "../workbench-session";

type Projection = Awaited<ReturnType<typeof loadSpaceSessionProjection>>;
const repository = vi.hoisted(() => ({
  load: vi.fn<(spaceId: string) => Promise<Projection>>(),
  markSpaceOpened: vi.fn<(spaceId: string) => Promise<void>>(),
}));
vi.mock("../../session/space-projection", () => ({ loadSpaceSessionProjection: repository.load }));
vi.mock("../space-operations", () => ({ markSpaceOpened: repository.markSpaceOpened }));

const SESSION_KEY = "knovum.workspace.workbench";
const emptyProjection: Projection = {
  nodes: [],
  scenes: [],
  relations: [],
  editSnapshot: { editRevision: 0, structure: [], layouts: [], relationStyles: [] },
};
const researchProjection: Projection = {
  nodes: [
    {
      id: "brief",
      type: "note",
      initialPosition: { x: 0, y: 0 },
      data: { kind: "note", title: "brief", tiptapContent: { type: "doc" }, plainText: "" },
    },
  ],
  scenes: [
    {
      id: "scene-1",
      spaceId: "research",
      title: "Evidence",
      thumbnailPath: null,
      viewportX: 120,
      viewportY: 40,
      viewportZoom: 0.75,
      orderIndex: 0,
      focusStructureNodeIds: ["content-brief"],
      relationKinds: ["supports"],
      relationStatuses: ["active"],
      queryFilter: {},
      createdAt: 1,
      updatedAt: 1,
    },
  ],
  relations: [],
  editSnapshot: {
    editRevision: 2,
    structure: [
      { id: "sec-1", kind: "section", parentId: null, order: 0, title: "Sources", locked: false },
      { id: "grp-1", kind: "group", parentId: "sec-1", order: 0, title: "Group A", locked: false },
      { id: "content-brief", kind: "content", nodeId: "brief", parentId: "grp-1", order: 0, locked: false },
    ],
    layouts: [],
    relationStyles: [],
  },
};
const inbox = createTestSpace();
const research = createTestSpace({ id: "research", kind: "space", title: "Research" });

function workbench(spaces: Space[] = [inbox]) {
  const state = createSpaceState();
  state.spaces.value = spaces;
  const opened: string[] = [];
  const context = createSpaceSessionContext(state, {
    open: (tab, node) => opened.push(`${tab.kind}:${node.id}`),
    release: () => {},
    canDiscard: () => false,
  });
  context.captureOverview = () => undefined;
  return { context, opened };
}

const save = (session: WorkbenchSession) => localStorage.setItem(SESSION_KEY, JSON.stringify(session));
const strip = (context: SpaceSessionContext) =>
  context.tabs.value.map((tab) => (tab.kind === "space" ? tab.spaceId : `${tab.kind}:${tab.nodeId}`));
const spaceTab = (spaceId: string) => ({ kind: "space" as const, spaceId });

describe("restoreWorkbench", () => {
  beforeEach(() => {
    repository.load
      .mockReset()
      .mockImplementation(async (spaceId) => (spaceId === "research" ? researchProjection : emptyProjection));
    repository.markSpaceOpened.mockReset().mockResolvedValue();
  });
  afterEach(() => localStorage.clear());

  it("reopens tabs, per-Space view context, and inspector view", async () => {
    const { context, opened } = workbench([inbox, research]);
    save({
      tabs: [
        spaceTab("inbox"),
        spaceTab("research"),
        { kind: "document", spaceId: "research", nodeId: "brief" },
      ],
      activeIndex: 2,
      rightSidebarView: "inspect",
      spaceViews: {
        research: {
          groupScopePath: ["grp-1", "deleted-group"],
          activeSceneId: "scene-1",
          outlineExpandedIds: ["sec-1", "content-brief", "missing"],
        },
      },
    });
    await restoreWorkbench(context, inbox);
    expect(strip(context)).toEqual(["inbox", "research", "document:brief"]);
    expect(context.activeTab.value).toMatchObject({ kind: "document", nodeId: "brief" });
    expect(context.rightSidebarView.value).toBe("inspect");
    expect(context.groupScopePath.value).toEqual(["grp-1"]);
    expect(context.activeSceneId.value).toBe("scene-1");
    expect(context.canvasFocusIds.value).toEqual(["brief"]);
    expect(context.relationKindFilter.value).toEqual(["supports"]);
    expect(context.outlineExpandedIds.value).toEqual(["sec-1"]);
    expect([...context.runtime.retained.keys()]).toEqual(["inbox"]);
    expect(opened).toEqual(["document:brief"]);
  });

  it("opens only Inbox when startup restore is disabled, drops unavailable tabs, and consumes bad sessions", async () => {
    const plans = createTestSpace({ id: "plans", kind: "space", title: "Plans", archivedAt: 5 });
    const disabled = workbench([inbox, research]);
    save({
      tabs: [spaceTab("inbox"), spaceTab("research")],
      activeIndex: 1,
      rightSidebarView: "inspect",
      spaceViews: { research: { activeSceneId: "scene-1", groupScopePath: ["grp-1"] } },
    });
    await restoreWorkbench(disabled.context, inbox, false);
    expect(strip(disabled.context)).toEqual(["inbox"]);
    expect(disabled.context.rightSidebarView.value).toBe("ask");
    expect(localStorage.getItem(SESSION_KEY)).toBeNull();

    const filtered = workbench([inbox, research, plans]);
    save({
      tabs: [
        spaceTab("research"),
        spaceTab("inbox"),
        spaceTab("plans"),
        { kind: "document", spaceId: "research", nodeId: "removed" },
        { kind: "video", spaceId: "research", nodeId: "brief" },
      ],
      activeIndex: 2,
      rightSidebarView: "ask",
      spaceViews: {},
    });
    await restoreWorkbench(filtered.context, inbox);
    expect(strip(filtered.context)).toEqual(["research", "inbox"]);
    expect(filtered.context.activeSpaceId.value).toBe("inbox");
    expect(filtered.opened).toEqual([]);

    save({ tabs: [spaceTab("inbox")], activeIndex: 0, rightSidebarView: "ask", spaceViews: {} });
    repository.load.mockRejectedValueOnce(new Error("database unavailable"));
    await expect(restoreWorkbench(workbench().context, inbox)).rejects.toThrow("database unavailable");
    expect(localStorage.getItem(SESSION_KEY)).toBeNull();

    localStorage.setItem(SESSION_KEY, JSON.stringify({ tabs: [{ kind: "space" }], activeIndex: 4 }));
    const invalid = workbench().context;
    await restoreWorkbench(invalid, inbox);
    expect(strip(invalid)).toEqual(["inbox"]);
  });
});
