import { indexStructureSnapshot } from "../../organization/structure/structure-context";
import { projectStructure, structureBoundsInParent } from "../../organization/structure/structure-projection";
import type { CanvasNodeLayoutV1, SpaceEditorDocument } from "../contract/canvas-document";
import { transformStructurePoint } from "./structure-transform";
export const SECTION_COLLAPSED_SIZE = { width: 320, height: 64 } as const;

/** World frames, UI order and scale are one-way projections of local geometry and sibling order. */
export function projectEditorLayouts(document: SpaceEditorDocument, expanded = false): CanvasNodeLayoutV1[] {
  const context = indexStructureSnapshot(document);
  const projection = projectStructure(context);
  return [...projection.paintOrder].map(([id, zIndex]) => {
    const layout = context.layouts.get(id)!;
    const world = projection.transforms.get(id)!;
    const bounds =
      layout.kind === "group"
        ? structureBoundsInParent(
            context,
            projection,
            (context.tree.children.get(id) ?? []).map((item) => item.id),
            id,
          )!
        : {
            x: 0,
            y: 0,
            ...(layout.kind === "section" && layout.collapsed && !expanded
              ? SECTION_COLLAPSED_SIZE
              : layout.size),
          };
    const point = transformStructurePoint(world, bounds);
    return {
      nodeId: id,
      zIndex,
      contentScale: layout.kind === "content" ? world.scale : 1,
      frame: {
        ...point,
        width: bounds.width * world.scale,
        height: bounds.height * world.scale,
        rotation: world.rotation,
      },
      sizeAuthority: layout.kind === "content" ? layout.sizeAuthority : "frame",
      ...(layout.kind !== "group" && layout.appearance ? { appearance: layout.appearance } : {}),
      ...(layout.kind === "content" && layout.fileSubpath ? { fileSubpath: layout.fileSubpath } : {}),
    };
  });
}
