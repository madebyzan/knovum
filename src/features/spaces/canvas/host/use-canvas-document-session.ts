import { shallowRef } from "vue";

import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { CanvasDocumentV1 } from "../contract/canvas-document";
import { createCanvasDocument, reconcileCanvasDocument } from "../document/canvas-document-sync";
import { registerSpaceSessionParticipant } from "../editor-session";
import {
  acquireCanvasPersistenceLease,
  registerCanvasMutationTarget,
} from "../persistence/canvas-persistence-coordinator";
import { canvasStateRepository } from "../persistence/canvas-state-repository";
import { CanvasSnapshotSaveQueue } from "../persistence/snapshot-save-queue";
import { rebaseCanvasDocument } from "./canvas-document-rebase";
import { isCurrentCanvasSession, requireCurrentCanvasSession } from "./canvas-session-identity";
import {
  commitCanvasSessionPersistence,
  releaseCanvasSessionPersistence,
} from "./canvas-session-persistence";

interface CanvasDocumentSessionOptions {
  isDraft(): boolean;
  nodes(): readonly SpaceCanvasNode[];
  reportError(error: unknown): void;
  spaceId(): string;
}

export function useCanvasDocumentSession(options: CanvasDocumentSessionOptions) {
  const document = shallowRef<CanvasDocumentV1>();
  const surfaceKey = shallowRef(0);
  let saveQueue: CanvasSnapshotSaveQueue | undefined;
  let persistenceLease: ReturnType<typeof acquireCanvasPersistenceLease> | undefined;
  let unregisterMutationTarget: (() => void) | undefined;
  let committedDocument: CanvasDocumentV1 | undefined;
  let pendingDocument: CanvasDocumentV1 | undefined;
  let pendingBaseDocument: CanvasDocumentV1 | undefined;
  let transactionTail = Promise.resolve();
  let transactionCount = 0;
  let generation = 0;
  const detachSessionCommit = registerSpaceSessionParticipant({
    commit: () => commitCanvasSessionPersistence(saveQueue, transactionTail, options.reportError),
    phase: "persistence",
  });

  async function commitTransaction<T extends { document: CanvasDocumentV1; revision: number }>(
    commit: (baseRevision: number) => Promise<T>,
  ) {
    const queue = saveQueue;
    const transactionGeneration = generation;
    const transactionSpaceId = options.spaceId();
    const expectedSession = {
      generation: transactionGeneration,
      queue,
      spaceId: transactionSpaceId,
    };
    if (!queue) throw new Error("Canvas persistence is not ready");
    if (transactionCount === 0) pendingBaseDocument = document.value;
    transactionCount += 1;
    const result = transactionTail.then(() =>
      queue.transaction(async (baseRevision) => {
        requireCurrentCanvasSession(expectedSession, currentSessionIdentity());
        const committed = await commit(baseRevision);
        if (!isCurrentCanvasSession(expectedSession, currentSessionIdentity())) return committed;
        const nextDocument =
          pendingDocument && pendingBaseDocument
            ? rebaseCanvasDocument(pendingBaseDocument, committed.document, pendingDocument)
            : committed.document;
        if (transactionCount > 1) {
          pendingBaseDocument = committed.document;
          pendingDocument = nextDocument;
        } else {
          pendingBaseDocument = undefined;
          pendingDocument = undefined;
        }
        committedDocument = nextDocument;
        document.value = nextDocument;
        if (nextDocument !== committed.document) queue.schedule(() => nextDocument);
        return { ...committed, document: nextDocument };
      }),
    );
    transactionTail = result.then(
      () => {
        transactionCount -= 1;
        return undefined;
      },
      () => {
        transactionCount -= 1;
        if (isCurrentCanvasSession(expectedSession, currentSessionIdentity()) && pendingDocument) {
          const pending = pendingDocument;
          pendingDocument = undefined;
          if (!options.isDraft()) saveQueue?.schedule(() => pending);
        }
        return undefined;
      },
    );
    const committed = await result;
    requireCurrentCanvasSession(expectedSession, currentSessionIdentity());
    return committed;
  }

  async function start() {
    const currentGeneration = ++generation;
    const previousQueue = saveQueue;
    const previousLease = persistenceLease;
    const previousTransactions = transactionTail;
    unregisterMutationTarget?.();
    unregisterMutationTarget = undefined;
    const spaceId = options.spaceId();
    saveQueue = undefined;
    committedDocument = undefined;
    pendingDocument = undefined;
    pendingBaseDocument = undefined;
    document.value = undefined;
    previousLease?.release(
      releaseCanvasSessionPersistence(previousQueue, previousTransactions, options.reportError),
    );
    persistenceLease = acquireCanvasPersistenceLease(spaceId);
    await persistenceLease.wait();
    if (currentGeneration !== generation) return;
    const stored = await canvasStateRepository.load(spaceId);
    if (currentGeneration !== generation) return;
    const initial = stored
      ? reconcileCanvasDocument(stored.document, options.nodes())
      : createCanvasDocument(options.nodes());
    saveQueue = new CanvasSnapshotSaveQueue({
      spaceId,
      persistence: canvasStateRepository,
      initialState: stored,
      reportError: options.reportError,
    });
    unregisterMutationTarget = registerCanvasMutationTarget(spaceId, async (mutation) => {
      if (currentGeneration !== generation || !document.value) {
        throw new Error("Canvas mutation target is no longer active");
      }
      documentChanged(mutation(document.value));
    });
    document.value = initial;
    surfaceKey.value += 1;
    if (!options.isDraft() && (!stored || initial !== stored.document)) saveQueue.schedule(() => initial);
  }

  function synchronizeNodes(nodes: readonly SpaceCanvasNode[]) {
    if (!document.value) return;
    const next = reconcileCanvasDocument(document.value, nodes);
    if (next !== document.value) documentChanged(next);
  }

  function documentChanged(next: CanvasDocumentV1) {
    document.value = next;
    if (next === committedDocument) {
      committedDocument = undefined;
      return;
    }
    if (transactionCount > 0) {
      pendingDocument = next;
      return;
    }
    // Draft Spaces have no database row yet; their viewport stays in memory until materialization.
    if (!options.isDraft()) saveQueue?.schedule(() => next);
  }

  function dispose() {
    generation += 1;
    unregisterMutationTarget?.();
    unregisterMutationTarget = undefined;
    persistenceLease?.release(
      releaseCanvasSessionPersistence(saveQueue, transactionTail, options.reportError),
    );
    persistenceLease = undefined;
    saveQueue = undefined;
    detachSessionCommit();
  }

  function currentSessionIdentity() {
    return { generation, queue: saveQueue, spaceId: options.spaceId() };
  }

  return { commitTransaction, dispose, document, documentChanged, start, surfaceKey, synchronizeNodes };
}
