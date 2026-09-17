import type { CanvasDocumentV1 } from "../contract/canvas-document";
import type {
  CanvasHistoryEntry,
  CanvasPersistenceHistory,
  CanvasPersistenceMutationResult,
} from "../contract/canvas-editor";
import { contentAddedBetween, removeCanvasContent, restoreCanvasContent } from "../document/canvas-content";

interface CanvasPersistenceHistoryOptions {
  before: CanvasDocumentV1;
  after: CanvasDocumentV1;
  persistence: CanvasPersistenceHistory;
  current(): CanvasDocumentV1;
  commit(
    document: CanvasDocumentV1,
    transition: CanvasPersistenceHistory["undo"],
  ): Promise<CanvasPersistenceMutationResult & { document: CanvasDocumentV1 }>;
  adopt(document: CanvasDocumentV1): void;
}

export function createCanvasPersistenceHistory(options: CanvasPersistenceHistoryOptions): CanvasHistoryEntry {
  return {
    undo: () => replay(options.after, options.before, options.persistence.undo),
    redo: () => replay(options.before, options.after, options.persistence.redo),
  };

  async function replay(
    source: CanvasDocumentV1,
    target: CanvasDocumentV1,
    transition: CanvasPersistenceHistory["undo"],
  ) {
    const document = applyContentTransition(options.current(), source, target);
    const result = await options.commit(document, transition);
    options.adopt(result.document);
  }
}

function applyContentTransition(
  current: CanvasDocumentV1,
  source: CanvasDocumentV1,
  target: CanvasDocumentV1,
) {
  const removed = contentAddedBetween(target, source);
  const added = contentAddedBetween(source, target);
  const withoutRemoved = removeCanvasContent(current, new Set(removed.nodes.map(({ nodeId }) => nodeId)));
  return restoreCanvasContent(withoutRemoved, target, new Set(added.nodes.map(({ nodeId }) => nodeId)));
}
