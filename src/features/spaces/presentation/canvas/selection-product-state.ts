import { isSpaceNodeOpenable } from "../../canvas/selection-actions";
import { isSpaceSemanticNode } from "../../knowledge/model/semantic-node";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { isSpaceManagedAssetPayload } from "../../model/nodes/capabilities";

export interface CanvasSelectionProductState {
  allLocked: boolean;
  askScope: "section" | "selection";
  canConnect: boolean;
  canOpen: boolean;
  opensSystemPreview: boolean;
  hasSectionMembership: boolean;
  content: SpaceCanvasNode[];
  section?: SpaceCanvasNode;
}

export function canvasSelectionProductState(
  nodes: readonly SpaceCanvasNode[],
  selectedIds: readonly string[],
): CanvasSelectionProductState {
  const selected = new Set(selectedIds);
  const selection = nodes.filter((node) => selected.has(node.id));
  const section = selection.find((node) => node.data.kind === "section");
  const content = selection.filter((node) => node.data.kind !== "section");
  const semantic = content.filter(isSpaceSemanticNode);
  return {
    allLocked: selection.length > 0 && selection.every((node) => Boolean(node.data.locked)),
    askScope: section ? "section" : "selection",
    canConnect: !section && content.length === 2 && semantic.length === 2,
    canOpen: selection.length === 1 && isSpaceNodeOpenable(selection[0]),
    opensSystemPreview:
      selection.length === 1 && Boolean(selection[0] && isSpaceManagedAssetPayload(selection[0].data)),
    hasSectionMembership: content.some(({ data }) => Boolean(data.sectionId)),
    content,
    ...(section ? { section } : {}),
  };
}
