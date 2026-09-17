import type { CanvasSnapshotSaveQueue } from "../persistence/snapshot-save-queue";

export async function commitCanvasSessionPersistence(
  queue: CanvasSnapshotSaveQueue | undefined,
  pendingTransactions: Promise<void>,
  reportError: (error: unknown) => void,
) {
  try {
    await flushCanvasSessionPersistence(queue, pendingTransactions);
  } catch (error) {
    reportError(error);
    throw error;
  }
}

export async function releaseCanvasSessionPersistence(
  queue: CanvasSnapshotSaveQueue | undefined,
  pendingTransactions: Promise<void>,
  reportError: (error: unknown) => void,
) {
  try {
    await flushCanvasSessionPersistence(queue, pendingTransactions);
  } catch (error) {
    reportError(error);
  }
}

async function flushCanvasSessionPersistence(
  queue: CanvasSnapshotSaveQueue | undefined,
  pendingTransactions: Promise<void>,
) {
  await pendingTransactions;
  await queue?.flush();
}
