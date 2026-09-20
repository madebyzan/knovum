import type { SpaceEditorDocument } from "../contract/canvas-document";
import type {
  CanvasHistoryEntry,
  CanvasPersistenceHistory,
  CanvasPersistenceMutationResult,
} from "../contract/canvas-editor";
import { replayEditorTransition } from "../document/editor-history-transition";
interface CanvasPersistenceHistoryOptions {
  before: SpaceEditorDocument;
  after: SpaceEditorDocument;
  persistence: CanvasPersistenceHistory;
  current(): SpaceEditorDocument;
  commit(
    document: SpaceEditorDocument,
    transition: CanvasPersistenceHistory["undo"],
    baseDocument: SpaceEditorDocument,
  ): Promise<CanvasPersistenceMutationResult & { document: SpaceEditorDocument }>;
  adopt(document: SpaceEditorDocument): void;
}
export function createCanvasPersistenceHistory(options: CanvasPersistenceHistoryOptions): CanvasHistoryEntry {
  async function replay(
    before: SpaceEditorDocument,
    after: SpaceEditorDocument,
    transition: CanvasPersistenceHistory["undo"],
  ) {
    const base = options.current();
    const result = await options.commit(replayEditorTransition(base, before, after), transition, base);
    options.adopt(result.document);
  }
  return {
    undo: () => replay(options.after, options.before, options.persistence.undo),
    redo: () => replay(options.before, options.after, options.persistence.redo),
  };
}
