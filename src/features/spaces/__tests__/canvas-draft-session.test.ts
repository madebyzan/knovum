import { flushPromises } from "@vue/test-utils";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import type { SaveCanvasStateRequest, StoredCanvasState } from "../canvas/contract/canvas-persistence";
import { flushSpaceSessionPersistence, runCanvasSessionTransition } from "../canvas/editor-session";
import { useCanvasDocumentSession } from "../canvas/host/use-canvas-document-session";
import { deferred, emptyCanvasDocument } from "../testing/canvas-fixtures";
import { createCanvasPersistenceFixture } from "../testing/canvas-persistence-fixture";

const repository = vi.hoisted(() => ({
  load: vi.fn<(spaceId: string) => Promise<StoredCanvasState | null>>(),
  save: vi.fn<(request: SaveCanvasStateRequest) => Promise<number>>(),
}));
vi.mock("../canvas/persistence/canvas-state-repository", () => ({ canvasStateRepository: repository }));

let fixture: ReturnType<typeof createCanvasPersistenceFixture>;
let session: ReturnType<typeof useCanvasDocumentSession>;
const ensurePersisted = vi.fn<() => Promise<void>>();
const reportError = vi.fn<(error: unknown) => void>();

beforeEach(() => {
  vi.useFakeTimers();
  vi.resetAllMocks();
  fixture = createCanvasPersistenceFixture();
  repository.load.mockImplementation(fixture.persistence.load);
  repository.save.mockImplementation(fixture.persistence.save);
  ensurePersisted.mockImplementation(async () => {
    fixture.spaces.add("draft");
  });
  session = useCanvasDocumentSession({
    nodes: () => [],
    spaceId: () => "draft",
    persistence: () => "draft",
    ensurePersisted,
    reportError,
  });
});
afterEach(async () => {
  session.dispose();
  await flushPromises();
  vi.clearAllTimers();
  vi.useRealTimers();
});

describe("draft canvas persistence lifecycle", () => {
  it("never reads or saves an empty draft during autosave, tab transition or disposal", async () => {
    await session.start();
    session.documentChanged({ ...emptyCanvasDocument, viewport: { x: 80, y: 120, zoom: 1.5 } });
    await vi.advanceTimersByTimeAsync(1_000);
    const activated = vi.fn<() => Promise<void>>(async () => {});
    await runCanvasSessionTransition(async () => {
      await flushSpaceSessionPersistence();
      await activated();
    });
    session.dispose();
    await flushPromises();
    expect(activated).toHaveBeenCalledOnce();
    expect(ensurePersisted).not.toHaveBeenCalled();
    expect(repository.load).not.toHaveBeenCalled();
    expect(repository.save).not.toHaveBeenCalled();
    expect(reportError).not.toHaveBeenCalled();
  });

  it("materializes once before concurrent first commits and preserves the live viewport", async () => {
    const parent = deferred<void>();
    ensurePersisted.mockImplementation(async () => {
      await parent.promise;
      fixture.spaces.add("draft");
    });
    await session.start();
    const key = session.surfaceKey.value;
    const document = { ...emptyCanvasDocument, viewport: { x: 80, y: 120, zoom: 1.5 } };
    session.documentChanged(document);
    const commit = vi.fn(async (baseRevision: number) => ({
      document,
      revision: await repository.save({ spaceId: "draft", baseRevision, document }),
    }));
    const first = session.commitTransaction(commit);
    const second = session.commitTransaction(commit);
    const promotion = session.persist();
    await flushPromises();
    expect(ensurePersisted).toHaveBeenCalledOnce();
    expect(repository.save).not.toHaveBeenCalled();
    parent.resolve(undefined);
    await Promise.all([first, second, promotion]);
    await flushSpaceSessionPersistence();
    expect(commit.mock.calls.map(([revision]) => revision)).toEqual([1, 2]);
    expect(fixture.states.get("draft")).toEqual({ revision: 3, document });
    expect(session.surfaceKey.value).toBe(key);
    expect(repository.load).not.toHaveBeenCalled();
  });

  it("promotes an externally materialized draft without remounting or resetting its document", async () => {
    await session.start();
    const key = session.surfaceKey.value;
    const document = { ...emptyCanvasDocument, viewport: { x: 30, y: 40, zoom: 2 } };
    session.documentChanged(document);
    fixture.spaces.add("draft");
    await Promise.all([session.persist(), session.persist()]);
    await flushSpaceSessionPersistence();
    expect(session.document.value).toBe(document);
    expect(session.surfaceKey.value).toBe(key);
    expect(repository.save).toHaveBeenCalledOnce();
    expect(fixture.states.get("draft")).toEqual({ revision: 1, document });
  });

  it("propagates parent creation failure without turning an empty draft flush into a write", async () => {
    ensurePersisted.mockRejectedValue(new Error("Space creation failed"));
    await session.start();
    const commit = vi.fn(async () => ({ document: emptyCanvasDocument, revision: 1 }));
    await expect(session.commitTransaction(commit)).rejects.toThrow("Space creation failed");
    await flushSpaceSessionPersistence();
    expect(commit).not.toHaveBeenCalled();
    expect(repository.save).not.toHaveBeenCalled();
    expect(repository.load).not.toHaveBeenCalled();
  });

  it("does not open storage when disposal supersedes pending materialization", async () => {
    const parent = deferred<void>();
    ensurePersisted.mockReturnValue(parent.promise);
    await session.start();
    const commit = vi.fn(async () => ({ document: emptyCanvasDocument, revision: 1 }));
    const result = session.commitTransaction(commit);
    const rejected = expect(result).rejects.toThrow("Canvas transaction target changed");
    await flushPromises();
    session.dispose();
    parent.resolve(undefined);
    await rejected;
    expect(commit).not.toHaveBeenCalled();
    expect(repository.save).not.toHaveBeenCalled();
    expect(repository.load).not.toHaveBeenCalled();
  });

  it("keeps real revision conflicts visible and never reloads or overwrites them", async () => {
    await session.start();
    await session.persist();
    await flushSpaceSessionPersistence();
    fixture.states.set("draft", { document: emptyCanvasDocument, revision: 2 });
    session.documentChanged({ ...emptyCanvasDocument, viewport: { x: 10, y: 20, zoom: 1 } });
    await expect(flushSpaceSessionPersistence()).rejects.toThrow("Canvas state revision conflict");
    expect(repository.save).toHaveBeenCalledTimes(2);
    expect(repository.load).not.toHaveBeenCalled();
    expect(fixture.states.get("draft")).toEqual({ document: emptyCanvasDocument, revision: 2 });
    expect(reportError).toHaveBeenCalledOnce();
  });

  it("retains viewport edits during materialization before allowing a tab transition", async () => {
    const parent = deferred<void>();
    ensurePersisted.mockImplementation(async () => {
      await parent.promise;
      fixture.spaces.add("draft");
    });
    await session.start();
    const transaction = session.commitTransaction(async (baseRevision) => ({
      document: emptyCanvasDocument,
      revision: await repository.save({ spaceId: "draft", baseRevision, document: emptyCanvasDocument }),
    }));
    await flushPromises();
    const viewport = { x: 200, y: 300, zoom: 2 };
    session.documentChanged({ ...emptyCanvasDocument, viewport });
    const transition = runCanvasSessionTransition(flushSpaceSessionPersistence);
    parent.resolve(undefined);
    await Promise.all([transaction, transition]);
    expect(fixture.states.get("draft")?.document.viewport).toEqual(viewport);
    expect(fixture.states.get("draft")?.revision).toBe(3);
  });
});
