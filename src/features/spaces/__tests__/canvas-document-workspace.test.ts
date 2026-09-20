import { describe, expect, it, vi } from "vitest";

import type { CanvasDocumentV1 } from "../canvas/contract/canvas-document";
import type { SaveCanvasStateRequest } from "../canvas/contract/canvas-persistence";
import { CanvasDocumentWorkspace } from "../canvas/host/canvas-document-workspace";
import { CanvasSnapshotSaveQueue } from "../canvas/persistence/snapshot-save-queue";

function fixture() {
  const document: CanvasDocumentV1 = {
    version: 1,
    viewport: { x: 0, y: 0, zoom: 1 },
    nodes: [
      {
        nodeId: "a",
        frame: { x: 0, y: 0, width: 100, height: 80, rotation: 0 },
        sizeAuthority: "frame",
        zIndex: 0,
      },
    ],
  };
  const save = vi.fn<(request: SaveCanvasStateRequest) => Promise<number>>(async () => 2);
  const publish = vi.fn<(document: CanvasDocumentV1) => void>();
  const owner = { active: true };
  const queue = new CanvasSnapshotSaveQueue({
    spaceId: "space-a",
    initialState: { document, revision: 0 },
    persistence: { load: vi.fn(async () => ({ document, revision: 0 })), save },
    reportError: vi.fn(),
  });
  const workspace = new CanvasDocumentWorkspace(document, queue, {
    isActive: () => owner.active,
    publish,
  });
  return { workspace, document, save, publish, owner };
}

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((finish) => {
    resolve = finish;
  });
  return { promise, resolve };
}

function moved(document: CanvasDocumentV1, x: number): CanvasDocumentV1 {
  return { ...document, nodes: document.nodes.map((node) => ({ ...node, frame: { ...node.frame, x } })) };
}

describe("Space-owned document workspace", () => {
  it("keeps a successful database acknowledgement when its renderer is detached", async () => {
    const { workspace, document, owner, publish } = fixture();
    const pending = deferred<{ document: CanvasDocumentV1; revision: number }>();
    const persist = vi.fn(() => pending.promise);
    const operation = workspace.commit(persist);
    await vi.waitFor(() => expect(persist).toHaveBeenCalledWith(0));
    owner.active = false;
    pending.resolve({ document: moved(document, 90), revision: 1 });
    await expect(operation).resolves.toMatchObject({ revision: 1 });
    expect(publish).not.toHaveBeenCalled();
    expect(workspace.document.nodes[0]?.frame.x).toBe(90);
    await workspace.flush();
  });

  it("rejects queued old-Space work before a second database call", async () => {
    const { workspace, document, owner } = fixture();
    const pending = deferred<{ document: CanvasDocumentV1; revision: number }>();
    const firstPersist = vi.fn(() => pending.promise);
    const secondPersist = vi.fn(async () => ({ document, revision: 2 }));
    const first = workspace.commit(firstPersist);
    const second = workspace.commit(secondPersist);
    const rejected = expect(second).rejects.toThrow("Canvas workspace changed during persistence");
    await vi.waitFor(() => expect(firstPersist).toHaveBeenCalledOnce());
    owner.active = false;
    pending.resolve({ document, revision: 1 });
    await expect(first).resolves.toMatchObject({ revision: 1 });
    await rejected;
    expect(secondPersist).not.toHaveBeenCalled();
    await workspace.flush();
  });

  it("does not make a successor's camera wait on the old Space transaction", async () => {
    const first = fixture();
    const second = fixture();
    const pending = deferred<{ document: CanvasDocumentV1; revision: number }>();
    const persist = vi.fn(() => pending.promise);
    const operation = first.workspace.commit(persist);
    await vi.waitFor(() => expect(persist).toHaveBeenCalledOnce());
    first.owner.active = false;
    const camera = { ...second.document, viewport: { x: 80, y: 35, zoom: 2 } };
    second.workspace.change(camera);
    await second.workspace.flush();
    expect(second.save).toHaveBeenCalledWith(expect.objectContaining({ document: camera }));
    pending.resolve({ document: first.document, revision: 1 });
    await operation;
  });

  it("persists the captured camera at the committed revision even after detachment", async () => {
    const { workspace, document, owner, save } = fixture();
    const pending = deferred<{ document: CanvasDocumentV1; revision: number }>();
    const persist = vi.fn(() => pending.promise);
    const operation = workspace.commit(persist);
    await vi.waitFor(() => expect(persist).toHaveBeenCalledOnce());
    workspace.change({ ...document, viewport: { x: 300, y: 40, zoom: 1.5 } });
    owner.active = false;
    pending.resolve({ document: moved(document, 120), revision: 1 });
    await operation;
    await workspace.flush();
    expect(save).toHaveBeenCalledWith(
      expect.objectContaining({
        baseRevision: 1,
        document: expect.objectContaining({ viewport: { x: 300, y: 40, zoom: 1.5 } }),
      }),
    );
    expect(save.mock.calls[0]?.[0].document.nodes[0]?.frame.x).toBe(120);
  });
});
