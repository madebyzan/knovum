import { expect, it, onTestFinished, vi } from "vitest";

import { ApplicationLeaveCancelled } from "../../../shared/contracts/application-leave";
import { createSpaceNode } from "../canvas/node";
import { createSpaceNotePayload } from "../document-editing/tiptap-content";
import { createEditorSurfaceActions } from "../workspace/editor-surface-actions";
import {
  prepareEditorSessionClose,
  registerSpaceSessionParticipant,
} from "../workspace/session-participants";
import { activateSpaceWorkItem, transitionActiveSpaceSession } from "../workspace/space-transition";
import { selectSpaceWorkItem } from "../workspace/work-items";
import { activateTestEditor, createTestSessionContext } from "./support/session-context";

it("opens distinct document tabs, reuses the target and returns to the retained canvas", async () => {
  const context = createTestSessionContext();
  context.nodes.value = ["a", "b"].map((id) =>
    createSpaceNode(createSpaceNotePayload(id), { x: 0, y: 0 }, id),
  );
  const actions = createEditorSurfaceActions(context);
  await actions.openDocumentEditor("a");
  const first = context.editorTabs.value[0]!;
  await actions.openDocumentEditor("b");
  await actions.openDocumentEditor("a");
  expect(context.editorTabs.value).toHaveLength(2);
  expect(context.activeWorkItems.value.get("inbox")).toBe(first.id);
  await actions.returnToCanvas();
  expect(context.surface.value.kind).toBe("canvas");
  expect(context.editorTabs.value).toHaveLength(2);
});

it("restores selected work items across Spaces and library navigation", () => {
  const context = createTestSessionContext();
  const a = activateTestEditor(context, "document", "a");
  context.workspaceDestination.value = "library";
  expect(context.surface.value.kind).toBe("library");
  context.activeSpaceId.value = "other";
  const b = activateTestEditor(context, "video", "b");
  context.activeSpaceId.value = "inbox";
  expect(context.surface.value).toEqual({ kind: "document", nodeId: "a" });
  expect(context.activeWorkItems.value.get("other")).toBe(b.id);
  expect(context.activeWorkItems.value.get("inbox")).toBe(a.id);
});

it("closes the active tab to its right neighbour without preparing another editor", async () => {
  const context = createTestSessionContext();
  const a = activateTestEditor(context, "document", "a");
  const b = activateTestEditor(context, "video", "b");
  selectSpaceWorkItem(context, "inbox", a.id);
  const saveA = vi.fn<() => Promise<void>>().mockResolvedValue();
  const saveB = vi.fn<() => Promise<void>>().mockResolvedValue();
  for (const [tab, commit] of [
    [a, saveA],
    [b, saveB],
  ] as const)
    onTestFinished(
      registerSpaceSessionParticipant({
        editorOwner: { spaceId: "inbox", tabId: tab.id, commitOnSwitch: tab.kind === "document" },
        commit,
      }),
    );
  await createEditorSurfaceActions(context).closeEditorTab(a.id);
  expect(saveA).toHaveBeenCalledOnce();
  expect(saveB).not.toHaveBeenCalled();
  expect(context.activeWorkItems.value.get("inbox")).toBe(b.id);
});

it("commits documents on navigation without requesting a video close decision", async () => {
  const context = createTestSessionContext();
  const document = activateTestEditor(context, "document", "document");
  const video = activateTestEditor(context, "video", "video");
  const save = vi.fn<() => Promise<void>>().mockResolvedValue();
  const close = vi.fn<() => Promise<void>>().mockRejectedValue(new ApplicationLeaveCancelled());
  onTestFinished(
    registerSpaceSessionParticipant({
      editorOwner: { spaceId: "inbox", tabId: document.id, commitOnSwitch: true },
      commit: save,
    }),
  );
  onTestFinished(
    registerSpaceSessionParticipant({
      editorOwner: { spaceId: "inbox", tabId: video.id, commitOnSwitch: false },
      commit: close,
    }),
  );
  await activateSpaceWorkItem(context, document.id);
  expect(save).toHaveBeenCalledOnce();
  expect(close).not.toHaveBeenCalled();
  await expect(transitionActiveSpaceSession(context, async () => {}, "close")).rejects.toBeInstanceOf(
    ApplicationLeaveCancelled,
  );
  expect(context.editorTabs.value).toHaveLength(2);
});

it("does not finalize an earlier discard when a later editor cancels batch close", async () => {
  const discard = vi.fn<() => void>();
  onTestFinished(
    registerSpaceSessionParticipant({
      editorOwner: { spaceId: "space", tabId: "first", commitOnSwitch: false },
      commit: async () => {},
      finalize: discard,
    }),
  );
  onTestFinished(
    registerSpaceSessionParticipant({
      editorOwner: { spaceId: "space", tabId: "second", commitOnSwitch: false },
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
  onTestFinished(
    registerSpaceSessionParticipant({
      editorOwner: { spaceId: "inbox", tabId: tab.id, commitOnSwitch: true },
      commit: async () => {
        throw failure;
      },
    }),
  );
  await expect(createEditorSurfaceActions(context).closeEditorTab(tab.id)).rejects.toBe(failure);
  expect(context.activeWorkItems.value.get("inbox")).toBe(tab.id);
  expect(context.editorTabs.value).toHaveLength(1);
});
