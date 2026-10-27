import { describe, expect, it, vi } from "vitest";

import { createSpaceHistoryState } from "../history/history-state";
import {
  beginSpaceMutation,
  closeSpaceMutationAdmission,
  waitForSpaceMutations,
} from "../history/mutation-barrier";
import { recordSpaceHistory, redoSpaceHistory, undoSpaceHistory } from "../history/stack";
import { emptySpaceContent, switchSpaceSession } from "../workspace/retained-space-sessions";
import { findSpaceCanvasTab } from "../workspace/workbench-tab";
import { selectWorkbenchTab } from "../workspace/workbench-tab-state";
import { createTestSessionContext, openTestSpaceTab } from "./support/session-context";

describe("space history pending mutations", () => {
  it("does not start undo while a canvas mutation is pending", async () => {
    const { canUndo, history } = createSpaceHistoryState();
    const undo = vi.fn<() => Promise<void>>(async () => {});
    recordSpaceHistory(history, { undo, redo: async () => {} });
    history.pendingMutations = 1;

    expect(canUndo.value).toBe(false);
    await undoSpaceHistory(history);

    expect(undo).not.toHaveBeenCalled();
    expect(history.undoStack).toHaveLength(1);
  });

  it("releases a session transition only after every mutation completes", async () => {
    const { history } = createSpaceHistoryState();
    const releaseFirst = beginSpaceMutation(history);
    const releaseSecond = beginSpaceMutation(history);
    let transitioned = false;
    const transition = waitForSpaceMutations(history).then(() => (transitioned = true));

    releaseFirst();
    await Promise.resolve();
    expect(transitioned).toBe(false);

    releaseSecond();
    await transition;
    expect(transitioned).toBe(true);
  });

  it("keeps history intact while mutation admission is closed", async () => {
    const { history } = createSpaceHistoryState();
    const undo = vi.fn<() => Promise<void>>(async () => {});
    const redo = vi.fn<() => Promise<void>>(async () => {});
    recordSpaceHistory(history, { undo, redo });
    const reopen = closeSpaceMutationAdmission(history);

    await expect(undoSpaceHistory(history)).rejects.toThrow("Space session is transitioning");
    expect(history.undoStack).toHaveLength(1);
    history.redoStack = history.undoStack;
    history.undoStack = [];
    await expect(redoSpaceHistory(history)).rejects.toThrow("Space session is transitioning");
    expect(history.redoStack).toHaveLength(1);
    expect(undo).not.toHaveBeenCalled();
    expect(redo).not.toHaveBeenCalled();
    reopen();
  });

  it("restores each open Space's undo stack after tab switches", async () => {
    const context = createTestSessionContext({ activeSpaceId: "space-a" });
    const { history } = context;
    const undoA = vi.fn<() => Promise<void>>().mockResolvedValue(undefined);
    const undoB = vi.fn<() => Promise<void>>().mockResolvedValue(undefined);
    const enter = (spaceId: string) => {
      switchSpaceSession(context, spaceId, emptySpaceContent());
      const tab = findSpaceCanvasTab(context.tabs.value, spaceId);
      if (tab) selectWorkbenchTab(context, tab.id);
      else openTestSpaceTab(context, spaceId);
    };

    recordSpaceHistory(history, { undo: undoA, redo: async () => {} });
    enter("space-b");
    recordSpaceHistory(history, { undo: undoB, redo: async () => {} });
    enter("space-a");

    await undoSpaceHistory(history);
    expect(undoA).toHaveBeenCalledOnce();
    expect(undoB).not.toHaveBeenCalled();

    enter("space-b");
    expect(history.undoStack).toHaveLength(1);
    await undoSpaceHistory(history);
    expect(undoB).toHaveBeenCalledOnce();
  });
});
