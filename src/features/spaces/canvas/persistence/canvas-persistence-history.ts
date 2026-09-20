import type { SpaceEditorDocument } from "../contract/canvas-document";
import type {
  CanvasHistoryEntry,
  CanvasPersistenceHistory,
  CanvasPersistenceMutationResult,
} from "../contract/canvas-editor";
import { contentAddedBetween, removeCanvasContent, restoreCanvasContent } from "../document/canvas-content";

interface CanvasPersistenceHistoryOptions {
  before: SpaceEditorDocument;
  after: SpaceEditorDocument;
  persistence: CanvasPersistenceHistory;
  current(): SpaceEditorDocument;
  commit(
    document: SpaceEditorDocument,
    transition: CanvasPersistenceHistory["undo"],
  ): Promise<CanvasPersistenceMutationResult & { document: SpaceEditorDocument }>;
  adopt(document: SpaceEditorDocument): void;
}

export function createCanvasPersistenceHistory(options: CanvasPersistenceHistoryOptions): CanvasHistoryEntry {
  return {
    undo: () => replay(options.after, options.before, options.persistence.undo),
    redo: () => replay(options.before, options.after, options.persistence.redo),
  };

  async function replay(
    source: SpaceEditorDocument,
    target: SpaceEditorDocument,
    transition: CanvasPersistenceHistory["undo"],
  ) {
    const document = applyContentTransition(options.current(), source, target);
    const result = await options.commit(document, transition);
    options.adopt(result.document);
  }
}

function applyContentTransition(
  current: SpaceEditorDocument,
  source: SpaceEditorDocument,
  target: SpaceEditorDocument,
) {
  const removed = contentAddedBetween(target, source);
  const added = contentAddedBetween(source, target);
  const withoutRemoved = removeCanvasContent(current, new Set(removed.nodes.map(({ nodeId }) => nodeId)));
  return restoreCanvasContent(withoutRemoved, target, new Set(added.nodes.map(({ nodeId }) => nodeId)));
}
