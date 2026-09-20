import { shallowRef, watch } from "vue";

import {
  indexStructure,
  requireStructureItem,
  structureAncestors,
} from "../../../model/structure/structure-index";
import { structureSelectionTarget } from "../../../organization/structure/structure-selection";
import type { SpaceEditorDocument } from "../../contract/canvas-document";
export function useStructureScope(
  document: () => SpaceEditorDocument,
  select: (ids: readonly string[]) => void,
) {
  const groupId = shallowRef<string | null>(null);
  const renameId = shallowRef<string>();
  function enter(id: string) {
    const tree = indexStructure(document().structure);
    const item = requireStructureItem(tree, id);
    if (item.kind !== "group" || item.locked || structureAncestors(tree, id).some((parent) => parent.locked))
      return;
    groupId.value = id;
    select([]);
  }
  function exit() {
    const id = groupId.value;
    if (!id) return;
    groupId.value =
      structureAncestors(indexStructure(document().structure), id).find((parent) => parent.kind === "group")
        ?.id ?? null;
    select([id]);
  }
  function setForSelection(ids: readonly string[]) {
    const tree = indexStructure(document().structure);
    const paths = ids
      .filter((id) => tree.items.has(id))
      .map((id) => structureAncestors(tree, id).filter((item) => item.kind === "group"));
    groupId.value =
      paths[0]?.find((item) => paths.every((path) => path.some((parent) => parent.id === item.id)))?.id ??
      null;
  }
  function resolveHit(id: string) {
    const tree = indexStructure(document().structure);
    const target = structureSelectionTarget(tree, id, groupId.value);
    if (target !== undefined) return target;
    groupId.value = null;
    return structureSelectionTarget(tree, id, null)!;
  }
  watch(document, (current) => {
    if (
      groupId.value &&
      !current.structure.items.some((item) => item.id === groupId.value && item.kind === "group")
    ) {
      groupId.value = null;
      select([]);
    }
    if (renameId.value && !current.structure.items.some((item) => item.id === renameId.value))
      renameId.value = undefined;
  });
  return { groupId, renameId, enter, exit, setForSelection, resolveHit };
}
