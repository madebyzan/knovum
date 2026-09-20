import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import type { CanvasPersistencePort } from "../canvas/contract/canvas-persistence";
import { createCanvasDocument } from "../canvas/document/canvas-document-sync";
import { flushSpaceSessionPersistence } from "../canvas/editor-session";
import { useCanvasDocumentSession } from "../canvas/host/use-canvas-document-session";
import { canvasNote, emptyCanvasDocument } from "../testing/canvas-fixtures";

const repository = vi.hoisted(() => ({
  load: vi.fn<CanvasPersistencePort["load"]>(),
  save: vi.fn<CanvasPersistencePort["save"]>(),
}));
vi.mock("../canvas/persistence/canvas-state-repository", () => ({ canvasStateRepository: repository }));

const sessions: ReturnType<typeof useCanvasDocumentSession>[] = [];
const reportError = vi.fn<(error: unknown) => void>();
let draft: boolean;
let revision: number;

beforeEach(() => {
  vi.useFakeTimers();
  draft = true;
  revision = 0;
  repository.load.mockResolvedValue(null);
  repository.save.mockImplementation(async (request) => {
    // Mirror the database's missing-Space and compare-and-swap rejection paths.
    if (draft || request.baseRevision !== revision) throw new Error("Canvas state revision conflict");
    revision += 1;
    return revision;
  });
});

afterEach(async () => {
  for (const session of sessions.splice(0)) session.dispose();
  await vi.runAllTimersAsync();
  vi.useRealTimers();
  vi.clearAllMocks();
});

function createSession() {
  const session = useCanvasDocumentSession({
    isDraft: () => draft,
    nodes: () => [],
    reportError,
    spaceId: () => "draft-space",
  });
  sessions.push(session);
  return session;
}

describe("draft canvas persistence", () => {
  it("does not save an empty draft on autosave, repeated tab flushes, or disposal", async () => {
    const session = createSession();
    await session.start();
    session.documentChanged({ ...emptyCanvasDocument, viewport: { x: 80, y: 120, zoom: 1.5 } });
    await vi.advanceTimersByTimeAsync(300);

    await expect(flushSpaceSessionPersistence()).resolves.toBeUndefined();
    await expect(flushSpaceSessionPersistence()).resolves.toBeUndefined();
    session.dispose();
    await vi.runAllTimersAsync();

    expect(repository.save).not.toHaveBeenCalled();
    expect(reportError).not.toHaveBeenCalled();
  });

  it("saves the latest viewport when the same draft becomes persisted without a node change", async () => {
    const session = createSession();
    await session.start();
    const latest = { ...emptyCanvasDocument, viewport: { x: 40, y: 60, zoom: 2 } };
    session.documentChanged(latest);
    await vi.advanceTimersByTimeAsync(300);

    draft = false;
    // CanvasSurfaceHost forwards the current document when its draft prop changes.
    session.documentChanged(session.document.value!);
    await vi.advanceTimersByTimeAsync(300);

    expect(repository.save).toHaveBeenCalledOnce();
    expect(repository.save).toHaveBeenCalledWith({
      spaceId: "draft-space",
      baseRevision: 0,
      document: latest,
    });
    expect(session.surfaceKey.value).toBe(1);
    expect(session.document.value).toBe(latest);
  });

  it("lets the first import materialize its Space before starting the revision chain", async () => {
    const session = createSession();
    await session.start();
    const viewport = { x: 24, y: 48, zoom: 1.5 };
    session.documentChanged({ ...emptyCanvasDocument, viewport: { x: 12, y: 12, zoom: 1 } });
    const imported = createCanvasDocument([canvasNote("imported")]);

    await session.commitTransaction(async (baseRevision) => {
      expect(baseRevision).toBe(0);
      expect(repository.save).not.toHaveBeenCalled();
      draft = false;
      session.documentChanged({ ...session.document.value!, viewport });
      revision = 1;
      return { document: imported, revision };
    });
    await flushSpaceSessionPersistence();

    expect(repository.save).toHaveBeenCalledOnce();
    expect(repository.save).toHaveBeenCalledWith({
      spaceId: "draft-space",
      baseRevision: 1,
      document: { ...imported, viewport },
    });
    expect(session.document.value?.nodes).toEqual(imported.nodes);
  });

  it("does not enqueue a draft snapshot after a transaction fails with a pending viewport", async () => {
    const session = createSession();
    await session.start();

    await expect(
      session.commitTransaction(async () => {
        session.documentChanged({ ...emptyCanvasDocument, viewport: { x: 10, y: 20, zoom: 2 } });
        throw new Error("Import failed before materialization");
      }),
    ).rejects.toThrow("Import failed before materialization");
    await expect(flushSpaceSessionPersistence()).resolves.toBeUndefined();
    await vi.advanceTimersByTimeAsync(300);

    expect(repository.save).not.toHaveBeenCalled();
    expect(reportError).not.toHaveBeenCalled();
  });

  it("still initializes the snapshot of an empty persisted Space", async () => {
    draft = false;
    await createSession().start();
    await flushSpaceSessionPersistence();

    expect(repository.save).toHaveBeenCalledOnce();
    expect(repository.save).toHaveBeenCalledWith({
      spaceId: "draft-space",
      baseRevision: 0,
      document: emptyCanvasDocument,
    });
  });

  it("releases an empty draft lease before remounting the same Space", async () => {
    const previous = createSession();
    await previous.start();
    previous.dispose();

    const next = createSession();
    await next.start();

    expect(next.surfaceKey.value).toBe(1);
    expect(repository.save).not.toHaveBeenCalled();
    expect(reportError).not.toHaveBeenCalled();
  });

  it("preserves real revision conflicts instead of retrying with a replacement revision", async () => {
    draft = false;
    revision = 4;
    repository.load.mockResolvedValue({ document: emptyCanvasDocument, revision: 3 });
    const session = createSession();
    await session.start();
    session.documentChanged({ ...emptyCanvasDocument, viewport: { x: 1, y: 2, zoom: 1 } });

    await expect(flushSpaceSessionPersistence()).rejects.toThrow("Canvas state revision conflict");
    await expect(flushSpaceSessionPersistence()).rejects.toThrow("Canvas state revision conflict");

    expect(repository.save).toHaveBeenCalledTimes(2);
    expect(repository.save.mock.calls.map(([request]) => request.baseRevision)).toEqual([3, 3]);
    expect(repository.load).toHaveBeenCalledOnce();
  });
});
