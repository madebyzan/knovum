import type { CanvasPackageCodec } from "../package/canvas-state-codec";
import { PACKAGE_ASSET_PREFIX } from "../package/contract";
import { parseSpaceEditorDocument } from "./contract/canvas-document";

export const canvasPackageCodec: CanvasPackageCodec = {
  validate: (source) => {
    parseSpaceEditorDocument(source.document);
  },
  remap(source, context) {
    const document = parseSpaceEditorDocument(source.document);
    const itemId = (id: string) => mapped(context.itemIds, id, "structure item");
    return {
      document: parseSpaceEditorDocument({
        ...document,
        structure: {
          version: 1,
          items: document.structure.items.map((item) => ({
            ...item,
            id: itemId(item.id),
            parentId: item.parentId === null ? null : itemId(item.parentId),
            ...(item.kind === "content" ? { nodeId: mapped(context.nodeIds, item.nodeId, "content") } : {}),
          })),
        },
        layout: {
          version: 1,
          items: document.layout.items.map((layout) => {
            const copy = structuredClone(layout);
            copy.itemId = itemId(layout.itemId);
            if (copy.kind !== "group" && copy.appearance?.background) {
              const path = copy.appearance.background;
              if (!path.startsWith(PACKAGE_ASSET_PREFIX))
                throw new Error("Package backgrounds must reference packaged resources");
              copy.appearance.background = mapped(
                context.assets,
                path.slice(PACKAGE_ASSET_PREFIX.length),
                "background asset",
              );
            }
            return copy;
          }),
        },
        relationStyles: (document.relationStyles ?? []).map((style) => ({
          ...style,
          relationId: mapped(context.relationIds, style.relationId, "relation"),
        })),
      }),
    };
  },
};

function mapped(mapping: ReadonlyMap<string, string>, id: string, kind: string) {
  const value = mapping.get(id);
  if (!value) throw new Error(`Package references an unknown ${kind}: ${id}`);
  return value;
}
