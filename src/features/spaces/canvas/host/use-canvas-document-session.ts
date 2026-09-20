import { shallowRef } from "vue";

import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { CanvasDocumentV1 } from "../contract/canvas-document";
import type { CanvasSpacePersistence } from "../contract/canvas-persistence";
import { createCanvasDocument, reconcileCanvasDocument } from "../document/canvas-document-sync";
import { registerSpaceSessionParticipant } from "../editor-session";
import { registerCanvasMutationTarget } from "../persistence/canvas-persistence-coordinator";
import type { CanvasSnapshotSaveQueue } from "../persistence/snapshot-save-queue";
import { createCanvasDocumentTransactions } from "./canvas-document-transactions";
import { requireCurrentCanvasSession } from "./canvas-session-identity";
import { commitCanvasSessionPersistence } from "./canvas-session-persistence";
import { openCanvasSessionStorage } from "./canvas-session-storage";

interface CanvasDocumentSessionOptions {
  nodes(): readonly SpaceCanvasNode[];
  reportError(error: unknown): void;
  spaceId(): string;
  persistence(): CanvasSpacePersistence;
  ensurePersisted(): Promise<void>;
}

export function useCanvasDocumentSession(options: CanvasDocumentSessionOptions) {
  const document = shallowRef<CanvasDocumentV1>();
  const surfaceKey = shallowRef(0);
  let saveQueue: CanvasSnapshotSaveQueue | undefined;
  let storage: ReturnType<typeof openCanvasSessionStorage> | undefined;
  let persistenceReady: Promise<void> | undefined;
  let unregisterMutationTarget: (() => void) | undefined;
  let generation = 0;
  let transactions = createCanvasDocumentTransactions({
    document,
    identity: currentSessionIdentity,
    ready: readyQueue,
  });
  const detachSessionCommit = registerSpaceSessionParticipant({
    async commit() {
      await transactions.pending;
      await persistenceReady;
      await commitCanvasSessionPersistence(saveQueue, transactions.pending, options.reportError);
    },
    phase: "persistence",
  });

  function commitTransaction<T extends { document: CanvasDocumentV1; revision: number }>(
    commit: (baseRevision: number) => Promise<T>,
  ) {
    return transactions.commit(commit);
  }

  async function readyQueue() {
    if (!saveQueue) await persist();
    if (!saveQueue) throw new Error("Canvas persistence is not ready");
    return saveQueue;
  }

  async function start() {
    generation += 1;
    unregisterMutationTarget?.();
    unregisterMutationTarget = undefined;
    storage?.release(transactions.pending);
    storage = undefined;
    persistenceReady = undefined;
    saveQueue = undefined;
    transactions = createCanvasDocumentTransactions({
      document,
      identity: currentSessionIdentity,
      ready: readyQueue,
    });
    document.value = undefined;
    if (options.persistence() === "draft") {
      publishDocument(createCanvasDocument(options.nodes()));
    } else {
      persistenceReady = openStorage("load");
      await persistenceReady;
    }
  }

  function persist() {
    if (persistenceReady) return persistenceReady;
    const expectedSession = currentSessionIdentity();
    const pending = Promise.resolve().then(async () => {
      await options.ensurePersisted();
      requireCurrentCanvasSession(expectedSession, currentSessionIdentity());
      await openStorage("create");
    });
    persistenceReady = pending;
    // A failed parent creation remains an action failure, not a permanently poisoned draft session.
    void pending.catch(() => {
      if (persistenceReady === pending && !storage) persistenceReady = undefined;
    });
    return pending;
  }

  async function openStorage(mode: "create" | "load") {
    const currentGeneration = generation;
    storage = openCanvasSessionStorage(options.spaceId(), mode, options.reportError);
    const { stored, queue } = await storage.ready;
    if (currentGeneration !== generation) return;
    const initial =
      document.value ??
      (stored
        ? reconcileCanvasDocument(stored.document, options.nodes())
        : createCanvasDocument(options.nodes()));
    saveQueue = queue;
    if (!document.value) publishDocument(initial);
    if (!stored || initial !== stored.document) queue.schedule(() => initial);
  }

  function publishDocument(initial: CanvasDocumentV1) {
    const currentGeneration = generation;
    unregisterMutationTarget = registerCanvasMutationTarget(options.spaceId(), async (mutation) => {
      if (currentGeneration !== generation || !document.value) {
        throw new Error("Canvas mutation target is no longer active");
      }
      documentChanged(mutation(document.value));
    });
    document.value = initial;
    surfaceKey.value += 1;
  }

  function synchronizeNodes(nodes: readonly SpaceCanvasNode[]) {
    if (!document.value) return;
    const next = reconcileCanvasDocument(document.value, nodes);
    if (next !== document.value) documentChanged(next);
  }

  function documentChanged(next: CanvasDocumentV1) {
    document.value = next;
    if (!transactions.consumesDocumentChange(next)) saveQueue?.schedule(() => next);
  }

  function dispose() {
    generation += 1;
    unregisterMutationTarget?.();
    unregisterMutationTarget = undefined;
    storage?.release(transactions.pending);
    storage = undefined;
    saveQueue = undefined;
    detachSessionCommit();
  }

  function currentSessionIdentity() {
    return { generation, spaceId: options.spaceId() };
  }

  return {
    commitTransaction,
    dispose,
    document,
    documentChanged,
    persist,
    start,
    surfaceKey,
    synchronizeNodes,
  };
}
