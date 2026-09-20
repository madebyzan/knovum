import type { ShallowRef } from "vue";

import type { CanvasDocumentV1 } from "../contract/canvas-document";
import type { CanvasSnapshotSaveQueue } from "../persistence/snapshot-save-queue";
import { rebaseCanvasDocument } from "./canvas-document-rebase";
import {
  isCurrentCanvasSession,
  requireCurrentCanvasSession,
  type CanvasSessionIdentity,
} from "./canvas-session-identity";

interface CanvasDocumentTransactionOptions {
  document: ShallowRef<CanvasDocumentV1 | undefined>;
  identity(): CanvasSessionIdentity;
  ready(): Promise<CanvasSnapshotSaveQueue>;
}

// Capture local changes from admission, including while a draft is being materialized.
// Rebase bookkeeping belongs to one generation, never to the next mounted Space.
export function createCanvasDocumentTransactions(options: CanvasDocumentTransactionOptions) {
  let committedDocument: CanvasDocumentV1 | undefined;
  let pendingDocument: CanvasDocumentV1 | undefined;
  let pendingBaseDocument: CanvasDocumentV1 | undefined;
  let tail = Promise.resolve();
  let count = 0;

  async function commit<T extends { document: CanvasDocumentV1; revision: number }>(
    persist: (baseRevision: number) => Promise<T>,
  ) {
    const expectedSession = options.identity();
    let queue: CanvasSnapshotSaveQueue | undefined;
    if (count === 0) pendingBaseDocument = options.document.value;
    count += 1;
    const result = tail.then(async () => {
      requireCurrentCanvasSession(expectedSession, options.identity());
      const activeQueue = await options.ready();
      queue = activeQueue;
      requireCurrentCanvasSession(expectedSession, options.identity());
      return activeQueue.transaction(async (baseRevision) => {
        requireCurrentCanvasSession(expectedSession, options.identity());
        const committed = await persist(baseRevision);
        if (!isCurrentCanvasSession(expectedSession, options.identity())) return committed;
        const nextDocument =
          pendingDocument && pendingBaseDocument
            ? rebaseCanvasDocument(pendingBaseDocument, committed.document, pendingDocument)
            : committed.document;
        if (count > 1) {
          pendingBaseDocument = committed.document;
          pendingDocument = nextDocument === committed.document ? undefined : nextDocument;
        } else {
          pendingBaseDocument = undefined;
          pendingDocument = undefined;
        }
        committedDocument = nextDocument;
        options.document.value = nextDocument;
        if (nextDocument !== committed.document) activeQueue.schedule(() => nextDocument);
        return { ...committed, document: nextDocument };
      });
    });
    tail = result.then(
      () => {
        count -= 1;
      },
      () => {
        count -= 1;
        if (isCurrentCanvasSession(expectedSession, options.identity()) && pendingDocument) {
          const pending = pendingDocument;
          pendingDocument = undefined;
          queue?.schedule(() => pending);
        }
      },
    );
    const committed = await result;
    requireCurrentCanvasSession(expectedSession, options.identity());
    return committed;
  }

  function consumesDocumentChange(next: CanvasDocumentV1) {
    if (next === committedDocument) {
      committedDocument = undefined;
      return true;
    }
    if (count > 0) {
      pendingDocument = next;
      return true;
    }
    return false;
  }

  return {
    commit,
    consumesDocumentChange,
    get pending() {
      return tail;
    },
  };
}
