import type { SpaceEditorDocument } from "../contract/canvas-document";
import type { CanvasPersistencePort, StoredCanvasState } from "../contract/canvas-persistence";

type CaptureCanvasDocument = () => SpaceEditorDocument;

interface CanvasSnapshotSaveQueueOptions {
  spaceId: string;
  persistence: CanvasPersistencePort;
  initialState: StoredCanvasState | null;
  delay?: number;
  reportError?: (error: unknown) => void;
}

export class CanvasSnapshotSaveQueue {
  private pendingCapture: CaptureCanvasDocument | undefined;
  private revision: number;
  private operationTail = Promise.resolve();
  private timer: ReturnType<typeof setTimeout> | undefined;

  private readonly delay: number;
  private readonly persistence: CanvasPersistencePort;
  private readonly reportError: (error: unknown) => void;
  private readonly spaceId: string;

  constructor(options: CanvasSnapshotSaveQueueOptions) {
    this.delay = options.delay ?? 300;
    this.persistence = options.persistence;
    this.reportError = options.reportError ?? (() => {});
    this.spaceId = options.spaceId;
    this.revision = options.initialState?.revision ?? 0;
  }

  schedule(captureDocument: CaptureCanvasDocument) {
    this.pendingCapture = captureDocument;
    clearTimeout(this.timer);
    this.timer = setTimeout(() => {
      this.timer = undefined;
      void this.flush().catch(this.reportError);
    }, this.delay);
  }

  get currentRevision() {
    return this.revision;
  }

  async flush() {
    clearTimeout(this.timer);
    this.timer = undefined;
    await this.enqueue(async () => this.persistPending());
  }

  async transaction<T extends { revision: number }>(commit: (baseRevision: number) => Promise<T>) {
    clearTimeout(this.timer);
    this.timer = undefined;
    return this.enqueue(async () => {
      await this.persistPending();
      const result = await commit(this.revision);
      this.revision = result.revision;
      return result;
    });
  }

  private enqueue<T>(operation: () => Promise<T>) {
    const result = this.operationTail.then(operation);
    this.operationTail = result.then(
      () => undefined,
      () => undefined,
    );
    return result;
  }

  private async persistPending() {
    while (this.pendingCapture) {
      const captureSnapshot = this.pendingCapture;
      this.pendingCapture = undefined;
      try {
        const document = captureSnapshot();
        // oxlint-disable-next-line no-await-in-loop -- Revisions are an ordered compare-and-swap chain.
        this.revision = await this.persistence.save({
          spaceId: this.spaceId,
          baseRevision: this.revision,
          document,
        });
      } catch (error) {
        this.pendingCapture ??= captureSnapshot;
        throw error;
      }
    }
  }
}
