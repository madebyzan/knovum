import type { SpaceEditorDocument } from "../contract/canvas-document";
import type { CanvasSnapshotSaveQueue } from "../persistence/snapshot-save-queue";
import { rebaseCanvasDocument } from "./canvas-document-rebase";

interface WorkspaceEvents {
  isActive(): boolean;
  publish(document: SpaceEditorDocument): void;
}

/** Owns the pending edit state of one mounted Space, never of its successor. */
export class CanvasDocumentWorkspace {
  private current: SpaceEditorDocument;
  private committedEcho: SpaceEditorDocument | undefined;
  private pendingDocument: SpaceEditorDocument | undefined;
  private pendingBase: SpaceEditorDocument | undefined;
  private pendingCount = 0;
  private transactions = Promise.resolve();

  constructor(
    initial: SpaceEditorDocument,
    private readonly queue: CanvasSnapshotSaveQueue,
    private readonly events: WorkspaceEvents,
  ) {
    this.current = initial;
  }

  get document() {
    return this.current;
  }

  change(document: SpaceEditorDocument) {
    if (!this.events.isActive()) return;
    this.current = document;
    this.events.publish(document);
    if (document === this.committedEcho) {
      this.committedEcho = undefined;
      return;
    }
    if (this.pendingCount) this.pendingDocument = document;
    else this.queue.schedule(() => document);
  }

  commit<T extends { document: SpaceEditorDocument; revision: number }>(
    persist: (revision: number) => Promise<T>,
  ): Promise<T> {
    if (!this.events.isActive()) {
      return Promise.reject(new Error("Canvas workspace changed during persistence"));
    }
    if (!this.pendingCount) this.pendingBase = this.current;
    this.pendingCount += 1;
    const result = this.transactions.then(async () => {
      // The save queue adopts the durable revision before any renderer publication.
      const committed = await this.queue.transaction((revision) => {
        if (!this.events.isActive()) {
          throw new Error("Canvas workspace changed during persistence");
        }
        return persist(revision);
      });
      const document =
        this.pendingDocument && this.pendingBase
          ? rebaseCanvasDocument(this.pendingBase, committed.document, this.pendingDocument)
          : committed.document;
      this.pendingBase = this.pendingCount > 1 ? committed.document : undefined;
      this.pendingDocument = this.pendingCount > 1 ? document : undefined;
      this.current = document;
      this.committedEcho = document;
      if (document !== committed.document) this.queue.schedule(() => document);
      if (this.events.isActive()) this.events.publish(document);
      // A finished transaction belongs to its original Space. Navigation cannot turn
      // its successful commit into a failure that would delete committed resources.
      return { ...committed, document };
    });
    this.transactions = result.then(
      () => {
        this.pendingCount -= 1;
      },
      () => {
        this.pendingCount -= 1;
        if (!this.pendingCount && this.pendingDocument) {
          const pending = this.pendingDocument;
          this.pendingDocument = undefined;
          this.pendingBase = undefined;
          this.queue.schedule(() => pending);
        }
      },
    );
    return result;
  }

  async flush() {
    await this.transactions;
    await this.queue.flush();
  }
}
