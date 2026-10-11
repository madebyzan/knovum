import type { AppCommand } from "../../../shared/contracts/app-command";
import { openSpaceNode } from "../canvas/selection-actions";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import { isSpaceManagedAssetPayload } from "../model/nodes/capabilities";

export function spacePreviewCommand(
  nodes: readonly SpaceCanvasNode[],
  selectedIds: readonly string[],
  title: string,
): AppCommand | undefined {
  const selected = new Set(selectedIds);
  const node = nodes.find(
    (candidate) => selected.has(candidate.id) && isSpaceManagedAssetPayload(candidate.data),
  );
  if (!node) return undefined;
  return {
    id: "spaces.system-preview",
    title,
    category: "canvas",
    run: () => openSpaceNode(node),
  };
}
