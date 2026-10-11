import { createStructureIndex } from "../../model/structure/structure-index";
import { createCanvasSelectionSnapshot } from "../document/canvas-selection-snapshot";
import { currentSelectionRoots } from "../selection/selection-roots";
import type { KonvaEditorPortOptions } from "./canvas-editor-port-options";

export function createCanvasEditorSelection(options: KonvaEditorPortOptions) {
  function roots() {
    const index = createStructureIndex(options.snapshot().structure);
    return currentSelectionRoots(options.selectedNodeIds(), options.selectedStructureNodeIds(), index);
  }

  function placements() {
    return roots().map(({ id }) => {
      const frame = options.structureBoundsById().get(id)!;
      return { id, x: frame.x, y: frame.y };
    });
  }

  function snapshot() {
    return createCanvasSelectionSnapshot({
      layouts: options.layouts(),
      nodeIds: options.selectedNodeIds(),
      relationIds: options.relationIds(),
      relationBoundsById: options.relationBoundsById(),
      structureNodeIds: options.selectedStructureNodeIds(),
      structureBoundsById: options.structureBoundsById(),
    });
  }

  function clear() {
    options.setSelection([]);
    options.setStructureSelection([]);
  }

  return { bounds: () => snapshot().bounds, clear, placements, roots, snapshot };
}
