import { computed } from "vue";

import { useCanvasEditorSession } from "../canvas/editor-session";
import { indexStructure, structureAncestors } from "../model/structure/structure-index";

export function useSelectedStructure() {
  const session = useCanvasEditorSession();
  const structure = computed(() => session.document.value?.structure);
  const selected = computed(() => {
    const [id, second] = session.selection.value.nodeIds;
    if (!structure.value || !id || second) return undefined;
    const tree = indexStructure(structure.value);
    const item = tree.items.get(id);
    return item
      ? { item, locked: item.locked || structureAncestors(tree, id).some((parent) => parent.locked) }
      : undefined;
  });
  return { selected, session };
}
