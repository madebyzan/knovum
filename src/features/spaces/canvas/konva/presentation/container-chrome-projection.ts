import { indexStructure, structureAncestors } from "../../../model/structure/structure-index";
import type { SpaceEditorDocument, CanvasNodeLayoutV1 } from "../../contract/canvas-document";
import type { CanvasViewport } from "../../contract/canvas-editor-port";
import type { CanvasScene } from "../scene/canvas-scene";
export function containerChromeProjection(
  document: SpaceEditorDocument,
  scene: CanvasScene,
  previews: ReadonlyMap<string, CanvasNodeLayoutV1>,
  viewport: CanvasViewport,
  visible: ReadonlySet<string>,
  selected: ReadonlySet<string>,
  renameId: string | undefined,
) {
  const tree = indexStructure(document.structure);
  const layouts = new Map(document.layout.items.map((item) => [item.itemId, item]));
  return scene.nodes.flatMap((node) => {
    const item = tree.items.get(node.node.id)!;
    if (item.kind === "content" || !visible.has(item.id)) return [];
    if (item.kind === "group" && !selected.has(item.id) && renameId !== item.id) return [];
    const layout = layouts.get(item.id)!;
    const frame = (previews.get(item.id) ?? node.layout).frame;
    const count =
      node.node.data.kind === "group" || node.node.data.kind === "section" ? node.node.data.contentCount : 0;
    return [
      {
        item,
        frame,
        count,
        ancestorLocked: structureAncestors(tree, item.id).some((parent) => parent.locked),
        collapsed: layout.kind === "section" && layout.collapsed,
        style: {
          left: `${viewport.x + frame.x * viewport.zoom + 8}px`,
          top: `${viewport.y + frame.y * viewport.zoom + (item.kind === "group" ? -32 : 4)}px`,
          width: `${Math.max(96, frame.width * viewport.zoom - 16)}px`,
        },
      },
    ];
  });
}
