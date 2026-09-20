import { uniq } from "es-toolkit";

import { currentCanvasEditor } from "../canvas/editor-session";
import { omit, type SelectionActionContext, updateSelected } from "./selection-mutations";
export function createSelectionActions(context: SelectionActionContext) {
  function editor() {
    const value = currentCanvasEditor();
    if (!value) throw new Error("The Space editor is not ready for an organizational command");
    return value;
  }
  return {
    addTag: async (ids: readonly string[], tag: string) => {
      const normalized = tag.trim();
      if (!normalized) return;
      await updateSelected(context, ids, (data) => ({
        ...data,
        tags: uniq([...(data.tags ?? []), normalized]),
      }));
    },
    removeTag: (ids: readonly string[], tag: string) =>
      updateSelected(context, ids, (data) => {
        const tags = data.tags?.filter((candidate) => candidate !== tag);
        return tags?.length ? { ...data, tags } : omit(data, "tags");
      }),
    setLocked: (ids: readonly string[], locked: boolean) =>
      editor().executeStructureCommand({ kind: "set-items-locked", ids: [...ids], locked }),
    deleteSelection: (ids: readonly string[], relationIds: readonly string[] = []) =>
      editor().deleteItems(ids, relationIds),
  };
}
