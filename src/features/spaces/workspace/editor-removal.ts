import { ancestorsOf, createStructureIndex } from "../model/structure/structure-index";
import { prepareEditorTabsClose } from "./editor-tab-lifecycle";
import type { SpaceSessionContext } from "./space-session-context";

export async function prepareEditorRemoval(context: SpaceSessionContext, rootIds: readonly string[]) {
  const spaceId = context.activeSpaceId.value;
  const roots = new Set(rootIds);
  const index = createStructureIndex(context.editSnapshot.value.structure);
  const tabs = context.editorTabs.value.filter((tab) => {
    if (tab.spaceId !== spaceId) return false;
    const structureId = index.structureNodeIdByNodeId.get(tab.nodeId);
    return Boolean(
      structureId &&
      (roots.has(structureId) || ancestorsOf(structureId, index).some(({ id }) => roots.has(id))),
    );
  });
  return prepareEditorTabsClose(context, tabs);
}
