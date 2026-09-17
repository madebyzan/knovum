import { uniq } from "es-toolkit";

import { createSectionActions } from "./section-actions";
import { omit, type SelectionActionContext, updateSelected } from "./selection-mutations";

export function createSelectionActions(context: SelectionActionContext) {
  async function addTag(ids: readonly string[], tag: string) {
    const normalized = tag.trim();
    if (!normalized) return;
    await updateSelected(context, ids, (data) => ({
      ...data,
      tags: uniq([...(data.tags ?? []), normalized]),
    }));
  }
  async function removeTag(ids: readonly string[], tag: string) {
    await updateSelected(context, ids, (data) => {
      const tags = data.tags?.filter((candidate) => candidate !== tag);
      return tags?.length ? { ...data, tags } : omit(data, "tags");
    });
  }
  async function setLocked(ids: readonly string[], locked: boolean) {
    await updateSelected(
      context,
      ids,
      (data) => (locked ? { ...data, locked: true } : omit(data, "locked")),
      { includeSections: true },
    );
  }
  return {
    addTag,
    removeTag,
    setLocked,
    ...createSectionActions(context),
    deleteSelection: (ids: readonly string[], relationIds?: readonly string[]) =>
      deleteSelection(context, ids, relationIds),
  };
}

async function deleteSelection(
  context: SelectionActionContext,
  ids: readonly string[],
  relationIds: readonly string[] = [],
) {
  const selected = new Set(ids);
  const sectionIds = new Set(
    context.nodes.value
      .filter((node) => selected.has(node.id) && node.data.kind === "section")
      .map((node) => node.id),
  );
  const survivors = context.nodes.value.filter(
    (node) => !selected.has(node.id) && node.data.sectionId && sectionIds.has(node.data.sectionId),
  );
  await context.recordHistoryMutation(async () => {
    await context.removeRelations(relationIds);
    await updateSelected(
      context,
      survivors.map((node) => node.id),
      (data) => omit(data, "sectionId"),
    );
    await context.deleteItems(ids);
  });
}
