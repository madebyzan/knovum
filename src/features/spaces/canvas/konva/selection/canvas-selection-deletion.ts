import { indexStructure } from "../../../model/structure/structure-index";
import { structureContentNodeIds } from "../../../organization/structure/structure-selection";
import { removeCanvasSelection } from "../../document/canvas-content";
import { createCanvasPersistenceHistory } from "../../persistence/canvas-persistence-history";
import type { KonvaEditorPortOptions } from "../canvas-editor-port-options";
export function createCanvasSelectionDeletion(options: KonvaEditorPortOptions) {
  return async (ids: readonly string[], relationIds: readonly string[], cut = false) => {
    if ((!ids.length && !relationIds.length) || (cut && !ids.length)) return;
    const before = options.model.document;
    const nodeIds = structureContentNodeIds(indexStructure(before.structure), ids);
    const content = new Set(nodeIds),
      selectedRelations = new Set(relationIds);
    for (const relation of options.relations())
      if (content.has(relation.sourceId) || content.has(relation.targetId))
        selectedRelations.add(relation.id);
    const document = removeCanvasSelection(before, new Set(ids), selectedRelations);
    await options.onHistoryTransaction(async () => {
      const request = { baseDocument: before, document, nodeIds, relationIds: [...selectedRelations] };
      const result = await (cut
        ? options.onCutSelection({ ...request, transfer: { document: before, itemIds: ids } })
        : options.onDeleteSelection(request));
      options.model.replace(result.document);
      options.onHistoryEntry(
        createCanvasPersistenceHistory({
          before,
          after: result.document,
          persistence: result.history,
          current: () => options.model.document,
          commit: options.onHistoryTransition,
          adopt: (next) => options.model.replace(next),
        }),
      );
    });
    options.setSelection([]);
  };
}
