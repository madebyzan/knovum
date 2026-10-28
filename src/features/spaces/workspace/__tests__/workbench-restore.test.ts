import { flushPromises } from "@vue/test-utils";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { nextTick, ref } from "vue";

import { createTestSpace } from "../../__tests__/support/session-context";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { Space } from "../../model/space";
import { createSpaceState } from "../../session/session-state";
import type { loadSpaceSessionProjection } from "../../session/space-projection";
import { createSpaceSessionContext } from "../../session/space-session-composition";
import type { SpaceSessionContext } from "../space-session-context";
import { restoreWorkbench } from "../workbench-restore-actions";
import { useWorkbenchSessionPersistence, type WorkbenchSession } from "../workbench-session";
import { createSpaceCanvasTab, createSpaceEditorTab, type SpaceWorkbenchTab } from "../workbench-tab";

type Projection = Awaited<ReturnType<typeof loadSpaceSessionProjection>>;
const repository = vi.hoisted(() => ({
  load: vi.fn<(spaceId: string) => Promise<Projection>>(),
  markSpaceOpened: vi.fn<(spaceId: string) => Promise<void>>(),
}));
vi.mock("../../session/space-projection", () => ({ loadSpaceSessionProjection: repository.load }));
vi.mock("../space-operations", () => ({ markSpaceOpened: repository.markSpaceOpened }));

const SESSION_KEY = "knovum.workspace.workbench";
const note = (id: string): SpaceCanvasNode => ({
  id,
  type: "note",
  initialPosition: { x: 0, y: 0 },
  data: { kind: "note", title: id, tiptapContent: { type: "doc" }, plainText: "" },
});
const nodesBySpace: Record<string, SpaceCanvasNode[]> = { inbox: [], research: [note("brief")], plans: [] };
const inbox = createTestSpace();

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

function save(session: WorkbenchSession) {
  localStorage.setItem(SESSION_KEY, JSON.stringify(session));
}

function strip(context: SpaceSessionContext) {
  return context.tabs.value.map((tab) => (tab.kind === "space" ? tab.spaceId : `${tab.kind}:${tab.nodeId}`));
}

describe("restoreWorkbench", () => {
  beforeEach(() => {
    repository.load.mockReset().mockImplementation(async (spaceId) => ({
      nodes: nodesBySpace[spaceId] ?? [],
      scenes: [],
      relations: [],
      editSnapshot: { editRevision: 0, structure: [], layouts: [], relationStyles: [] },
    }));
    repository.markSpaceOpened.mockReset().mockResolvedValue();
  });
  afterEach(() => localStorage.clear());

  it("opens only the Inbox when no session was saved", async () => {
    const { context } = workbench();

    await restoreWorkbench(context, inbox);

    expect(strip(context)).toEqual(["inbox"]);
    expect(context.activeSpaceId.value).toBe("inbox");
    expect(context.navigationHistory.value).toEqual([context.activeTabId.value]);
  });

  it("reopens the saved strip in order with its active tab and editor sessions", async () => {
    const research = createTestSpace({ id: "research", kind: "space", title: "Research" });
    const { context, opened } = workbench([inbox, research]);
    save({
      tabs: [
        { kind: "space", spaceId: "inbox" },
        { kind: "space", spaceId: "research" },
        { kind: "document", spaceId: "research", nodeId: "brief" },
      ],
      activeIndex: 2,
    });

    await restoreWorkbench(context, inbox);

    expect(strip(context)).toEqual(["inbox", "research", "document:brief"]);
    expect(context.activeTab.value).toMatchObject({ kind: "document", nodeId: "brief" });
    expect(context.nodes.value.map(({ id }) => id)).toEqual(["brief"]);
    expect([...context.runtime.retained.keys()]).toEqual(["inbox"]);
    expect(opened).toEqual(["document:brief"]);
    expect(repository.markSpaceOpened).toHaveBeenCalledExactlyOnceWith("research");
  });

  it("drops tabs whose Space or content is gone and then activates the Inbox", async () => {
    const research = createTestSpace({ id: "research", kind: "space", title: "Research" });
    const plans = createTestSpace({ id: "plans", kind: "space", title: "Plans", archivedAt: 5 });
    const { context, opened } = workbench([inbox, research, plans]);
    save({
      tabs: [
        { kind: "space", spaceId: "research" },
        { kind: "space", spaceId: "inbox" },
        { kind: "space", spaceId: "plans" },
        { kind: "document", spaceId: "research", nodeId: "removed" },
        { kind: "video", spaceId: "research", nodeId: "brief" },
        { kind: "space", spaceId: "deleted" },
      ],
      activeIndex: 2,
    });

    await restoreWorkbench(context, inbox);

    expect(strip(context)).toEqual(["research", "inbox"]);
    expect(context.activeSpaceId.value).toBe("inbox");
    expect([...context.runtime.retained.keys()]).toEqual(["research"]);
    expect(repository.load).not.toHaveBeenCalledWith("plans");
    expect(opened).toEqual([]);
  });

  it("consumes the saved session so a failed restore starts from the Inbox next time", async () => {
    save({ tabs: [{ kind: "space", spaceId: "inbox" }], activeIndex: 0 });
    repository.load.mockRejectedValueOnce(new Error("database unavailable"));

    await expect(restoreWorkbench(workbench().context, inbox)).rejects.toThrow("database unavailable");

    expect(localStorage.getItem(SESSION_KEY)).toBeNull();
  });

  it("ignores a saved value that is not a workbench session", async () => {
    localStorage.setItem(SESSION_KEY, JSON.stringify({ tabs: [{ kind: "space" }], activeIndex: 4 }));
    const { context } = workbench();

    await restoreWorkbench(context, inbox);

    expect(strip(context)).toEqual(["inbox"]);
    expect(localStorage.getItem(SESSION_KEY)).toBeNull();
  });
});

describe("useWorkbenchSessionPersistence", () => {
  afterEach(() => localStorage.clear());

  it("saves the strip once ready and leaves out the tabs of an unsaved draft", async () => {
    const draft = createTestSpace({ id: "draft", kind: "space", title: "Draft" });
    const canvas = createSpaceCanvasTab("inbox");
    const editor = createSpaceEditorTab("inbox", "video", "clip");
    const source = {
      ready: ref(false),
      tabs: ref<readonly SpaceWorkbenchTab[]>([canvas, createSpaceCanvasTab("draft"), editor]),
      activeTabId: ref<string | undefined>(editor.id),
      draftSpace: ref<Space | undefined>(draft),
    };
    useWorkbenchSessionPersistence(source);

    source.activeTabId.value = canvas.id;
    await nextTick();
    expect(localStorage.getItem(SESSION_KEY)).toBeNull();

    source.ready.value = true;
    await flushPromises();
    expect(JSON.parse(localStorage.getItem(SESSION_KEY)!)).toEqual({
      tabs: [
        { kind: "space", spaceId: "inbox" },
        { kind: "video", spaceId: "inbox", nodeId: "clip" },
      ],
      activeIndex: 0,
    });
  });
});
