import type { KonvaEditorPortOptions } from "../canvas-editor-port-options";

export function createCanvasSelectionDeletion(options: KonvaEditorPortOptions) {
  return async (ids: readonly string[], relationIds: readonly string[], cut = false) => {
    if ((!ids.length && !relationIds.length) || (cut && !ids.length)) return;
    const before = options.model.document;
    await options.onHistoryTransaction(async () => {
      if (cut) await options.onCopyNodes({ document: before, itemIds: ids });
      await options.model.executeNative({ kind: "delete-selection", ids, relationIds }, before);
    });
    options.setSelection([]);
  };
}
