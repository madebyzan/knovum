import { describe, expect, it, vi } from "vitest";

import { createSpaceHistoryState } from "../history/history-state";
import {
  beginSpaceMutation,
  closeSpaceMutationAdmission,
  waitForSpaceMutations,
} from "../history/mutation-barrier";
import { recordSpaceHistory, redoSpaceHistory, undoSpaceHistory } from "../history/stack";
import { switchSpaceHistory } from "../workspace/session-history";
import type { SpaceSessionContext } from "../workspace/space-session-context";

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
    const { history } = createSpaceHistoryState();
    const undoA = vi.fn<() => Promise<void>>().mockResolvedValue(undefined);
    const undoB = vi.fn<() => Promise<void>>().mockResolvedValue(undefined);
    const context = {
      activeSpaceId: { value: "space-a" },
      history,
      runtime: { openGeneration: 0, historyBySpace: new Map() },
    } as unknown as SpaceSessionContext;

    recordSpaceHistory(history, { undo: undoA, redo: async () => {} });
    switchSpaceHistory(context, "space-b");
    context.activeSpaceId.value = "space-b";
    recordSpaceHistory(history, { undo: undoB, redo: async () => {} });
    switchSpaceHistory(context, "space-a");
    context.activeSpaceId.value = "space-a";

    await undoSpaceHistory(history);
    expect(undoA).toHaveBeenCalledOnce();
    expect(undoB).not.toHaveBeenCalled();

    switchSpaceHistory(context, "space-b");
    expect(history.undoStack).toHaveLength(1);
    await undoSpaceHistory(history);
    expect(undoB).toHaveBeenCalledOnce();
  });
});
