import { planStructureCommand } from "../../organization/structure/commands/plan";
import type { SpaceEditorDocument } from "../contract/canvas-document";
export function removeCanvasContent(document: SpaceEditorDocument, ids: ReadonlySet<string>) {
  return ids.size
    ? { ...document, ...planStructureCommand(document, { kind: "delete-subtrees", ids: [...ids] }).after }
    : document;
}
export function removeCanvasSelection(
  document: SpaceEditorDocument,
  ids: ReadonlySet<string>,
  relationIds: ReadonlySet<string>,
) {
  const after = removeCanvasContent(document, ids);
  return {
    ...after,
    relationStyles: after.relationStyles?.filter((style) => !relationIds.has(style.relationId)) ?? [],
  };
}
