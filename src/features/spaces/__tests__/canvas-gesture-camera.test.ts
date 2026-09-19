import { describe, expect, it, vi } from "vitest";

import { movementHarness } from "../canvas/konva/testing/canvas-movement-harness";
import { deferred } from "../testing/canvas-fixtures";
import { movementScene } from "../testing/canvas-movement-fixture";

describe("committing geometry while the camera remains live", () => {
  it.each([false, true])(
    "keeps camera autosave free of uncommitted geometry (rejected=%s)",
    async (rejected) => {
      const harness = movementHarness(movementScene());
      const before = structuredClone(harness.model.document);
      const pending = deferred<void>();
      vi.mocked(harness.events.onCommitGesture).mockImplementationOnce(async (request) => {
        await pending.promise;
        if (rejected) throw new Error("revision conflict");
        return {
          document: request.document,
          revision: 1,
          history: { undo: async () => ({ revision: 2 }), redo: async () => ({ revision: 3 }) },
        };
      });

      const move = harness.session.nudge({ x: 100, y: 30 });
      await Promise.resolve();
      expect(harness.events.onCommitGesture).toHaveBeenCalledOnce();
      expect(harness.model.document).toEqual(before);
      expect(harness.documentChanged).not.toHaveBeenCalled();

      const viewport = { x: 120, y: 80, zoom: 1.5 };
      harness.model.setViewport(viewport);
      expect(harness.documentChanged).toHaveBeenCalledExactlyOnceWith({ ...before, viewport });
      const outcome = move.then(
        () => "committed",
        (error: unknown) => error,
      );
      pending.resolve(undefined);
      await expect(outcome).resolves.toEqual(rejected ? new Error("revision conflict") : "committed");

      expect(harness.model.document.viewport).toEqual(viewport);
      expect(harness.model.document.nodes.map(({ frame }) => frame.x)).toEqual(
        rejected ? [0, 1000, 40, 1080] : [100, 1100, 140, 1180],
      );
      expect(harness.documentChanged).toHaveBeenCalledTimes(1);
      expect(harness.events.onHistoryEntry).toHaveBeenCalledTimes(rejected ? 0 : 1);
      expect(harness.session.busy.value).toBe(false);
    },
  );

  it.each(["undo", "redo"] as const)("preserves camera changes while %s is committing", async (direction) => {
    const harness = movementHarness(movementScene());
    await harness.session.nudge({ x: 20, y: 0 });
    const entry = vi.mocked(harness.events.onHistoryEntry).mock.calls[0]![0];
    if (direction === "redo") await entry.undo();
    const before = structuredClone(harness.model.document);
    const pending = deferred<void>();
    vi.spyOn(harness.events, "onHistoryTransition").mockImplementationOnce(async (document, transition) => {
      await pending.promise;
      return { document, ...(await transition({ baseRevision: 1, document })) };
    });
    harness.documentChanged.mockClear();

    const replay = entry[direction]();
    expect(harness.model.document).toEqual(before);
    const viewport = { x: -200, y: 50, zoom: 0.75 };
    harness.model.setViewport(viewport);
    expect(harness.documentChanged).toHaveBeenCalledExactlyOnceWith({ ...before, viewport });
    pending.resolve(undefined);
    await replay;

    expect(harness.model.document.viewport).toEqual(viewport);
    expect(harness.model.document.nodes.map(({ frame }) => frame.x)).toEqual(
      direction === "undo" ? [0, 1000, 40, 1080] : [20, 1020, 60, 1100],
    );
    expect(harness.documentChanged).toHaveBeenCalledTimes(1);
  });
});
