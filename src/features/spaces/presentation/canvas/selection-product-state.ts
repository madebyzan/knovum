import { isSpaceNodeOpenable } from "../../canvas/selection-actions";
import { isSpaceSemanticNode } from "../../knowledge/model/semantic-node";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { isSpaceManagedAssetPayload } from "../../model/nodes/capabilities";
import { indexStructure } from "../../model/structure/structure-index";
import type { SpaceStructure } from "../../model/structure/structure-schema";
import { structureSelectionEligibility } from "../../organization/structure/structure-eligibility";
import { structureContentIds } from "../../organization/structure/structure-selection";

export function canvasSelectionProductState(
  nodes: readonly SpaceCanvasNode[],
  selectedIds: readonly string[],
  structure: SpaceStructure,
) {
  const state = structureSelectionEligibility(structure, selectedIds);
  const selected = new Set(state.roots.map((item) => item.id));
  const directContent = nodes.filter((node) => selected.has(node.id));
  const contentIds = new Set(structureContentIds(indexStructure(structure), [...selected]));
  const content = nodes.filter((node) => contentIds.has(node.id));
  const single = state.roots.length === 1 ? state.roots[0] : undefined;
  const section = single?.kind === "section" ? single : undefined;
  const group = single?.kind === "group" ? single : undefined;
  const data = single?.kind === "content" ? directContent[0]?.data : undefined;
  const textKind = data?.kind === "note" || data?.kind === "text" ? data.kind : undefined;
  return {
    ...state,
    section,
    group,
    content,
    directContent,
    textKind,
    allLocked: state.roots.length > 0 && state.roots.every((item) => item.locked),
    askScope: section ? ("section" as const) : ("selection" as const),
    canConnect:
      state.editable &&
      state.roots.length === 2 &&
      directContent.length === 2 &&
      directContent.every(isSpaceSemanticNode),
    canOpen: single?.kind === "content" && isSpaceNodeOpenable(directContent[0]),
    opensSystemPreview: Boolean(data && isSpaceManagedAssetPayload(data)),
    hasParent: state.roots.some((item) => item.parentId !== null),
  };
}
