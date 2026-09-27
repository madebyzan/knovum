import type { SpaceEditSnapshot } from "../../model/structure/schema/edit-snapshot";
import { createStructureIndex } from "../../model/structure/structure-index";
import { worldTransformFor } from "../../model/structure/world-transform";
import type { CanvasViewport } from "../contract/canvas-editor-port";
import type { CanvasDocumentV1 } from "../schema/document";

const DEFAULT_VIEWPORT: CanvasViewport = { x: 0, y: 0, zoom: 1 };

/** Konva adapter document projected from committed Structure/Layout; never persisted. */
export function projectStructureCanvasDocument(
  snapshot: SpaceEditSnapshot,
  viewport: CanvasViewport = DEFAULT_VIEWPORT,
): CanvasDocumentV1 {
  const index = createStructureIndex(snapshot.structure);
  const layouts = new Map(snapshot.layouts.map((layout) => [layout.structureNodeId, layout]));
  const orderedContentIds: string[] = [];

  function walk(parentId: string | null) {
    for (const structureNode of index.childrenByParent.get(parentId) ?? []) {
      if (structureNode.kind === "content") {
        orderedContentIds.push(structureNode.id);
        continue;
      }
      walk(structureNode.id);
    }
  }
  walk(null);

  return {
    version: 1,
    viewport: { ...viewport },
    nodes: orderedContentIds.map((structureNodeId, zIndex) => {
      const structureNode = index.structureById.get(structureNodeId);
      const layout = layouts.get(structureNodeId);
      if (structureNode?.kind !== "content" || layout?.kind !== "content")
        throw new Error(`Content structure is incomplete: ${structureNodeId}`);
      const world = worldTransformFor(structureNodeId, index, layouts);
      return Object.assign(
        {},
        {
          nodeId: structureNode.nodeId,
          frame: {
            x: world.x,
            y: world.y,
            width: layout.size.width * world.scale,
            height: layout.size.height * world.scale,
            rotation: world.rotation,
          },
          sizeAuthority: layout.sizeAuthority,
          zIndex,
        },
        layout.appearance ? { appearance: layout.appearance } : {},
        layout.fileSubpath ? { fileSubpath: layout.fileSubpath } : {},
      );
    }),
    ...(snapshot.relationStyles.length ? { relationStyles: snapshot.relationStyles } : {}),
  };
}
