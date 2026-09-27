import { compact } from "es-toolkit";

import { createStructureIndex } from "../../model/structure/structure-index";
import { canvasNodePlacements, createCanvasSelectionSnapshot } from "../document/canvas-selection-snapshot";
import { currentSelectionRoots } from "../selection/selection-roots";
import type { KonvaEditorPortOptions } from "./canvas-editor-port-options";

export function createCanvasEditorSelection(options: KonvaEditorPortOptions) {
  function roots() {
    const index = createStructureIndex(options.snapshot().structure);
    return currentSelectionRoots(options.selectedNodeIds(), options.selectedStructureNodeIds(), index);
  }

  function placements() {
    const structurePlacements = roots().flatMap(({ id }) => {
      const frame = options.structureBounds().get(id);
      return frame ? [{ id, x: frame.x, y: frame.y }] : [];
    });
    return structurePlacements.length
      ? structurePlacements
      : canvasNodePlacements(options.layouts(), options.selectedNodeIds());
  }

  function contentSnapshot() {
    return createCanvasSelectionSnapshot(
      options.layouts(),
      options.selectedNodeIds(),
      options.relationIds(),
      options.relationBounds(),
    );
  }

  function bounds() {
    const frames = compact(roots().map(({ id }) => options.structureBounds().get(id)));
    if (!frames.length) return contentSnapshot().bounds;
    const left = Math.min(...frames.map(({ x }) => x));
    const top = Math.min(...frames.map(({ y }) => y));
    const right = Math.max(...frames.map(({ x, width }) => x + width));
    const bottom = Math.max(...frames.map(({ y, height }) => y + height));
    return { x: left, y: top, width: right - left, height: bottom - top };
  }

  function snapshot() {
    const selectionBounds = bounds();
    const content = contentSnapshot();
    const structureMembers = compact(roots().map(({ id }) => options.structureBounds().get(id)));
    return {
      ...content,
      structureNodeIds: [...options.selectedStructureNodeIds()],
      members: structureMembers.length ? structureMembers : content.members,
      ...(selectionBounds ? { bounds: selectionBounds } : {}),
    };
  }

  function clear() {
    options.setSelection([]);
    options.setStructureSelection([]);
  }

  return { bounds, clear, placements, roots, snapshot };
}
