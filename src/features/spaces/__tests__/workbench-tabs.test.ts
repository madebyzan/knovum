import { expect, it, onTestFinished, vi } from "vitest";

import { ApplicationLeaveCancelled } from "../../../shared/contracts/application-leave";
import { createSpaceNode } from "../canvas/node";
import { createSpaceNotePayload } from "../document-editing/tiptap-content";
import {
  prepareEditorSessionClose,
  registerSpaceSessionParticipant,
} from "../workspace/session-participants";
import {
  closeOtherWorkbenchTabs,
  closeWorkbenchTab,
  closeWorkbenchTabsToRight,
} from "../workspace/workbench-close-actions";
import {
  activateWorkbenchTab,
  openEditorTab,
  openSpaceCanvas,
  reopenClosedWorkbenchTab,
} from "../workspace/workbench-navigation-actions";
import { selectWorkbenchTab } from "../workspace/workbench-tab-state";
import { activateTestEditor, createTestSessionContext } from "./support/session-context";

function sessionWithNotes(...nodeIds: string[]) {
  const context = createTestSessionContext();
  context.nodes.value = nodeIds.map((id) => createSpaceNode(createSpaceNotePayload(id), { x: 0, y: 0 }, id));
  return context;
}

function registerEditor(tabId: string, commit: () => Promise<void>) {
  onTestFinished(
    registerSpaceSessionParticipant({
      editorOwner: { spaceId: "inbox", tabId },
      commit,
    }),
  );
}

it("opens editor tabs right of the active tab, reuses the target and keeps the canvas tab", async () => {
  const context = sessionWithNotes("a", "b");
  const canvas = context.tabs.value[0]!;
  await openEditorTab(context, "document", "a");
  const first = context.editorTabs.value[0]!;
  await activateWorkbenchTab(context, canvas.id);
  await openEditorTab(context, "document", "b");
  await openEditorTab(context, "document", "a");

  expect(context.editorTabs.value).toHaveLength(2);
  expect(context.activeTabId.value).toBe(first.id);
  expect(context.tabs.value.map(({ id }) => id)).toEqual([
    canvas.id,
    context.editorTabs.value[0]!.id,
    first.id,
  ]);
  await openSpaceCanvas(context);
  expect(context.surface.value.kind).toBe("canvas");
  expect(context.tabs.value).toHaveLength(3);
});

it("derives the work surface from the active tab", async () => {
  const context = sessionWithNotes("a");
  const canvas = context.tabs.value[0]!;
  await openEditorTab(context, "document", "a");
  const document = context.editorTabs.value[0]!;

  expect(context.surface.value).toEqual({ kind: "document", nodeId: "a" });
  context.workspaceDestination.value = "library";
  expect(context.surface.value.kind).toBe("library");
  await activateWorkbenchTab(context, canvas.id);
  expect(context.surface.value.kind).toBe("canvas");
  await activateWorkbenchTab(context, document.id);
  expect(context.surface.value).toEqual({ kind: "document", nodeId: "a" });
});

it("closes the active tab to its right neighbour without preparing another editor", async () => {
  const context = createTestSessionContext();
  const a = activateTestEditor(context, "document", "a");
  const b = activateTestEditor(context, "video", "b");
  selectWorkbenchTab(context, a.id);
  const saveA = vi.fn<() => Promise<void>>().mockResolvedValue();
  const saveB = vi.fn<() => Promise<void>>().mockResolvedValue();
  registerEditor(a.id, saveA);
  registerEditor(b.id, saveB);

  await closeWorkbenchTab(context, a.id);

  expect(saveA).toHaveBeenCalledOnce();
  expect(saveB).not.toHaveBeenCalled();
  expect(context.activeTabId.value).toBe(b.id);
});

it("does not finalize an earlier discard when a later editor cancels batch close", async () => {
  const discard = vi.fn<() => void>();
  onTestFinished(
    registerSpaceSessionParticipant({
      editorOwner: { spaceId: "space", tabId: "first" },
      commit: async () => {},
      finalize: discard,
    }),
  );
  onTestFinished(
    registerSpaceSessionParticipant({
      editorOwner: { spaceId: "space", tabId: "second" },
      commit: async () => {
        throw new ApplicationLeaveCancelled();
      },
    }),
  );
  await expect(prepareEditorSessionClose({ spaceIds: ["space"] })).rejects.toBeInstanceOf(
    ApplicationLeaveCancelled,
  );
  expect(discard).not.toHaveBeenCalled();
});

it("retains the active tab and its list when its save fails", async () => {
  const context = createTestSessionContext();
  const tab = activateTestEditor(context, "document", "a");
  const failure = new Error("disk full");
  registerEditor(tab.id, async () => {
    throw failure;
  });

  await expect(closeWorkbenchTab(context, tab.id)).rejects.toBe(failure);

  expect(context.activeTabId.value).toBe(tab.id);
  expect(context.editorTabs.value).toHaveLength(1);
});

it("keeps the Inbox canvas tab open through every close action", async () => {
  const context = createTestSessionContext();
  const inbox = context.tabs.value[0]!;
  const document = activateTestEditor(context, "document", "a");

  await closeWorkbenchTab(context, inbox.id);
  expect(context.tabs.value).toHaveLength(2);

  await closeOtherWorkbenchTabs(context, document.id);
  expect(context.tabs.value.map(({ id }) => id)).toEqual([inbox.id, document.id]);

  await closeWorkbenchTabsToRight(context, inbox.id);
  expect(context.tabs.value.map(({ id }) => id)).toEqual([inbox.id]);
  expect(context.activeTabId.value).toBe(inbox.id);
});

it("reopens the most recently closed tab at its original position", async () => {
  const context = sessionWithNotes("a", "b");
  const inbox = context.tabs.value[0]!;
  await openEditorTab(context, "document", "a");
  const first = context.editorTabs.value[0]!;
  await openEditorTab(context, "document", "b");
  const second = context.editorTabs.value[1]!;

  await closeWorkbenchTab(context, first.id);
  expect(context.tabs.value.map(({ id }) => id)).toEqual([inbox.id, second.id]);

  await reopenClosedWorkbenchTab(context);

  expect(context.tabs.value.map(({ id }) => id)).toEqual([inbox.id, first.id, second.id]);
  expect(context.activeTabId.value).toBe(first.id);
  expect(context.closedTabs.value).toHaveLength(0);
});
