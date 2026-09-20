import { acquireCanvasPersistenceLease } from "../persistence/canvas-persistence-coordinator";
import { canvasStateRepository } from "../persistence/canvas-state-repository";
import { CanvasSnapshotSaveQueue } from "../persistence/snapshot-save-queue";
import { releaseCanvasSessionPersistence } from "./canvas-session-persistence";

// Only persisted Spaces own a storage lease and a revision queue. Promotion creates the first
// canvas state; opening an existing Space loads its authoritative revision instead.
export function openCanvasSessionStorage(
  spaceId: string,
  mode: "create" | "load",
  reportError: (error: unknown) => void,
) {
  const lease = acquireCanvasPersistenceLease(spaceId);
  const ready = lease.wait().then(async () => {
    const stored = mode === "load" ? await canvasStateRepository.load(spaceId) : null;
    const queue = new CanvasSnapshotSaveQueue({
      spaceId,
      persistence: canvasStateRepository,
      initialState: stored,
      reportError,
    });
    return { stored, queue };
  });
  return {
    ready,
    release(pendingTransactions: Promise<void>) {
      lease.release(
        ready.then(
          ({ queue }) => releaseCanvasSessionPersistence(queue, pendingTransactions, reportError),
          reportError,
        ),
      );
    },
  };
}
