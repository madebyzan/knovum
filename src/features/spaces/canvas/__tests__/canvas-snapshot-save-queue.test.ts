import { describe, expect, it, vi } from "vitest";

import type { CanvasPersistencePort } from "../contract/canvas-persistence";
import { CanvasSnapshotSaveQueue } from "../persistence/snapshot-save-queue";

describe("CanvasSnapshotSaveQueue", () => {
  it("flushes the latest accepted snapshot before disposal", async () => {
    vi.useFakeTimers();
    const save = vi.fn<CanvasPersistencePort["save"]>(async () => 8);
    const persistence: CanvasPersistencePort = { load: async () => null, save };
    const queue = new CanvasSnapshotSaveQueue({ spaceId: "space-1", persistence, initialState: null });

    const staleSnapshot = vi.fn<
      () => {
        version: 1;
        viewport: { x: number; y: number; zoom: number };
        nodes: [];
      }
    >(() => ({
      version: 1 as const,
      viewport: { x: 0, y: 0, zoom: 1 },
      nodes: [],
    }));
    const latestSnapshot = vi.fn<
      () => {
        version: 1;
        viewport: { x: number; y: number; zoom: number };
        nodes: [];
      }
    >(() => ({
      version: 1 as const,
      viewport: { x: 2, y: 3, zoom: 2 },
      nodes: [],
    }));
    queue.schedule(staleSnapshot);
    queue.schedule(latestSnapshot);

    expect(staleSnapshot).not.toHaveBeenCalled();
    expect(latestSnapshot).not.toHaveBeenCalled();
    await queue.flush();

    expect(staleSnapshot).not.toHaveBeenCalled();
    expect(latestSnapshot).toHaveBeenCalledOnce();
    expect(save).toHaveBeenCalledOnce();
    expect(save).toHaveBeenCalledWith({
      spaceId: "space-1",
      baseRevision: 0,
      document: { version: 1, viewport: { x: 2, y: 3, zoom: 2 }, nodes: [] },
    });
    vi.useRealTimers();
  });

  it("serializes revisions while retaining changes accepted during a save", async () => {
    let releaseFirstSave: (() => void) | undefined;
    const save = vi
      .fn<CanvasPersistencePort["save"]>()
      .mockImplementationOnce(
        () =>
          new Promise<number>((resolve) => {
            releaseFirstSave = () => resolve(4);
          }),
      )
      .mockResolvedValueOnce(5);
    const queue = emptyQueue(save);

    queue.schedule(() => ({ version: 1, viewport: { x: 0, y: 0, zoom: 1 }, nodes: [] }));
    await vi.waitFor(() => expect(save).toHaveBeenCalledOnce());
    queue.schedule(() => ({ version: 1, viewport: { x: 2, y: 3, zoom: 2 }, nodes: [] }));
    releaseFirstSave?.();
    await queue.flush();

    expect(save).toHaveBeenNthCalledWith(2, {
      spaceId: "space-1",
      baseRevision: 4,
      document: { version: 1, viewport: { x: 2, y: 3, zoom: 2 }, nodes: [] },
    });
  });

  it("retains an uncommitted snapshot after a persistence failure", async () => {
    const save = vi
      .fn<CanvasPersistencePort["save"]>()
      .mockRejectedValueOnce(new Error("revision conflict"))
      .mockResolvedValueOnce(3);
    const queue = emptyQueue(save);

    queue.schedule(() => ({ version: 1, viewport: { x: 0, y: 0, zoom: 1 }, nodes: [] }));
    await expect(queue.flush()).rejects.toThrow("revision conflict");
    await queue.flush();

    expect(save).toHaveBeenNthCalledWith(2, {
      spaceId: "space-1",
      baseRevision: 0,
      document: { version: 1, viewport: { x: 0, y: 0, zoom: 1 }, nodes: [] },
    });
  });

  it("continues the revision chain after an application transaction", async () => {
    const save = vi.fn<CanvasPersistencePort["save"]>(async () => 13);
    const queue = new CanvasSnapshotSaveQueue({
      spaceId: "space-1",
      persistence: { load: async () => null, save },
      initialState: {
        revision: 10,
        document: { version: 1, viewport: { x: 0, y: 0, zoom: 1 }, nodes: [] },
      },
    });

    await queue.transaction(async (baseRevision) => {
      expect(baseRevision).toBe(10);
      return { revision: 12 };
    });
    queue.schedule(() => ({ version: 1, viewport: { x: 4, y: 5, zoom: 1 }, nodes: [] }));
    await queue.flush();

    expect(save).toHaveBeenCalledWith(expect.objectContaining({ baseRevision: 12 }));
    expect(queue.currentRevision).toBe(13);
  });

  it("serializes snapshots scheduled while an application transaction is in flight", async () => {
    let finishTransaction: ((value: { revision: number }) => void) | undefined;
    const save = vi.fn<CanvasPersistencePort["save"]>(async () => 12);
    const queue = new CanvasSnapshotSaveQueue({
      spaceId: "space-1",
      persistence: { load: async () => null, save },
      initialState: {
        revision: 10,
        document: { version: 1, viewport: { x: 0, y: 0, zoom: 1 }, nodes: [] },
      },
      delay: 0,
    });
    const transaction = queue.transaction(
      () => new Promise<{ revision: number }>((resolve) => (finishTransaction = resolve)),
    );
    await vi.waitFor(() => expect(finishTransaction).toBeTypeOf("function"));
    queue.schedule(() => ({
      version: 1,
      viewport: { x: 7, y: 9, zoom: 1 },
      nodes: [],
    }));
    expect(save).not.toHaveBeenCalled();

    finishTransaction?.({ revision: 11 });
    await transaction;
    await queue.flush();

    expect(save).toHaveBeenCalledWith(expect.objectContaining({ baseRevision: 11 }));
    expect(queue.currentRevision).toBe(12);
  });
});

function emptyQueue(save: CanvasPersistencePort["save"]) {
  return new CanvasSnapshotSaveQueue({
    spaceId: "space-1",
    persistence: { load: async () => null, save },
    initialState: null,
    delay: 0,
  });
}
