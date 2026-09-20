import { nextTick } from "vue";

import { indexStructure, structureAncestors } from "../../../model/structure/structure-index";
import type { KonvaEditorPortOptions } from "../canvas-editor-port-options";
import { revealCanvasLayouts } from "./canvas-editor-navigation";
export function createStructureNavigation(options: KonvaEditorPortOptions) {
  function select(ids: readonly string[]) {
    options.setEditingScope(ids);
    options.setSelection(ids);
  }
  async function revealItems(ids: readonly string[], mode: "ensure-visible" | "focus") {
    if (!ids.length) {
      select([]);
      return;
    }
    const before = options.model.document;
    const tree = indexStructure(before.structure);
    const existing = ids.filter((id) => tree.items.has(id));
    const sections = new Set(
      existing.flatMap((id) =>
        structureAncestors(tree, id)
          .filter((item) => item.kind === "section")
          .map((item) => item.id),
      ),
    );
    if (sections.size)
      await options.model.mutate(
        (document) => ({
          ...document,
          layout: {
            version: 1,
            items: document.layout.items.map((item) =>
              item.kind === "section" && sections.has(item.itemId) ? { ...item, collapsed: false } : item,
            ),
          },
        }),
        false,
      );
    options.setEditingScope(existing);
    await nextTick();
    revealCanvasLayouts({ ...options, setViewport: options.animateViewport }, existing, mode);
  }
  return {
    select,
    reveal: (ids: readonly string[], mode: "ensure-visible" | "focus") => {
      void revealItems(ids, mode).catch(options.onError);
    },
  };
}
