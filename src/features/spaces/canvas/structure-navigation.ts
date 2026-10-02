import { createStructureIndex } from "../model/structure/structure-index";
import { groupScopePathFor, sectionAncestorOf } from "../model/structure/structure-queries";
import type { useSpaceStore } from "../session/space-store";
import type { getCanvasEditorSession } from "./editor-session";

type SpaceStore = Pick<
  ReturnType<typeof useSpaceStore>,
  "editSnapshot" | "executeSpaceEdit" | "groupScopePath"
>;
type EditorSession = Pick<
  ReturnType<typeof getCanvasEditorSession>,
  "selectNodes" | "selectStructureNodes" | "reveal"
>;

export async function revealStructureNode(store: SpaceStore, editor: EditorSession, structureNodeId: string) {
  const index = createStructureIndex(store.editSnapshot.structure);
  const structureNode = index.structureById.get(structureNodeId);
  if (!structureNode) throw new Error("Structure search result is no longer available");
  const section =
    structureNode.kind === "section" ? structureNode : sectionAncestorOf(structureNodeId, index);
  const sectionLayout = section
    ? store.editSnapshot.layouts.find((layout) => layout.structureNodeId === section.id)
    : undefined;
  if (section && sectionLayout?.kind === "section" && sectionLayout.collapsed) {
    await store.executeSpaceEdit({
      kind: "section.set-collapsed",
      structureNodeId: section.id,
      collapsed: false,
    });
  }
  store.groupScopePath = groupScopePathFor(structureNodeId, index);
  if (structureNode.kind === "content") editor.selectNodes([structureNode.nodeId]);
  else editor.selectStructureNodes([structureNodeId]);
  editor.reveal([structureNode.kind === "content" ? structureNode.nodeId : structureNodeId], "focus");
}

export async function revealContentNode(store: SpaceStore, editor: EditorSession, nodeId: string) {
  const index = createStructureIndex(store.editSnapshot.structure);
  const structureNodeId = index.nodeIdToStructureNodeId.get(nodeId);
  if (!structureNodeId) throw new Error("Search result is unavailable in the Space structure");
  await revealStructureNode(store, editor, structureNodeId);
}
