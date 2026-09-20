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
    const document = options.model.document;
    const tree = indexStructure(document.structure);
    const existing = ids.filter((id) => tree.items.has(id));
    const ancestors = new Set(
      existing.flatMap((id) =>
        structureAncestors(tree, id)
          .filter((item) => item.kind === "section")
          .map((item) => item.id),
      ),
    );
    for (const layout of document.layout.items) {
      if (layout.kind !== "section" || !layout.collapsed || !ancestors.has(layout.itemId)) continue;
      // Reveal changes presentation only; every write still uses a native command and revision.
      // oxlint-disable-next-line no-await-in-loop -- Each command adopts its committed revision before the next reveal.
      await options.model.execute(
        { kind: "set-section-collapsed", id: layout.itemId, collapsed: false },
        [],
        false,
      );
    }
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
