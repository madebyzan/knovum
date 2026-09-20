import { indexStructureSnapshot } from "../../organization/structure/structure-context";
import { assertStructureEditable } from "../../organization/structure/structure-selection";
import {
  parseSpaceEditorDocument,
  type CanvasNodeAppearance,
  type CanvasRelationColor,
  type SpaceEditorDocument,
} from "../contract/canvas-document";

export function setEditorAppearance(
  document: SpaceEditorDocument,
  id: string,
  patch: Partial<CanvasNodeAppearance>,
) {
  assertStructureEditable(indexStructureSnapshot(document).tree, [id]);
  const items = document.layout.items.map((layout) => {
    if (layout.itemId !== id) return layout;
    if (layout.kind === "group") throw new Error("Groups have no independent content appearance");
    const appearance = { ...layout.appearance, ...patch };
    if (patch.fill) delete appearance.customFill;
    if (patch.customFill) delete appearance.fill;
    for (const key of Object.keys(appearance) as (keyof CanvasNodeAppearance)[])
      if (appearance[key] === undefined) delete appearance[key];
    return { ...layout, appearance };
  });
  return parseSpaceEditorDocument({ ...document, layout: { version: 1, items } });
}
export function setEditorRelationColor(
  document: SpaceEditorDocument,
  relationId: string,
  color: CanvasRelationColor,
): SpaceEditorDocument {
  const retained = document.relationStyles?.filter((style) => style.relationId !== relationId) ?? [];
  return {
    ...document,
    relationStyles: color === "neutral" ? retained : [...retained, { relationId, color }],
  };
}
