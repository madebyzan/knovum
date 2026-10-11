import type { CanvasNodePlacement } from "../../canvas/contracts/canvas-editor-port";
import { copySpaceSelectionToClipboard } from "../../canvas/selection-actions";
import { createSpaceClipboardPayload } from "../../clipboard/payload";
import { useSpaceStore } from "../space-store";

export function createCanvasSelectionActions() {
  const store = useSpaceStore();

  function clipboardPayload(nodeIds: readonly string[]) {
    const structureNodeIds = new Set(store.editSnapshot.structure.map(({ id }) => id));
    const structureNodeIdByContentId = new Map(
      store.editSnapshot.structure.flatMap((structureNode) =>
        structureNode.kind === "content" ? [[structureNode.nodeId, structureNode.id] as const] : [],
      ),
    );
    const selectedStructureNodeIds = nodeIds.map((selectionId) => {
      const structureNodeId = structureNodeIds.has(selectionId)
        ? selectionId
        : structureNodeIdByContentId.get(selectionId);
      if (!structureNodeId)
        throw new Error(`Clipboard node is unavailable in the Space structure: ${selectionId}`);
      return structureNodeId;
    });
    return createSpaceClipboardPayload({
      snapshot: store.editSnapshot,
      nodes: store.nodes,
      relations: store.relations,
      selectedStructureNodeIds,
    });
  }

  return {
    copyNodes: (placements: readonly CanvasNodePlacement[]) =>
      copySpaceSelectionToClipboard(clipboardPayload(placements.map(({ id }) => id))),
  };
}
