import { describe, expect, it, vi } from "vitest";

import { createSpaceHistoryState } from "../history-state";
import {
  beginSpaceMutation,
  closeSpaceMutationAdmission,
  waitForSpaceMutations,
} from "../mutation-barrier";
import { recordSpaceHistory, redoSpaceHistory, undoSpaceHistory } from "../stack";

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
});
