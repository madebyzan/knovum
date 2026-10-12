import { i18n } from "../../../foundation/i18n/instance";
import type { SpaceEditorResource } from "../contracts/generated/SpaceEditorResource";
import type { SpaceEditSnapshot } from "../model/structure/schema/edit-snapshot";
import { createStructureIndex } from "../model/structure/structure-index";
import { contentNodeIdsWithin } from "../model/structure/structure-queries";
import type { SpaceEditorTab } from "./workbench-tab";

export function editorResources(tabs: readonly SpaceEditorTab[]): SpaceEditorResource[] {
  return tabs.map(({ spaceId, nodeId }) => ({ spaceId, nodeId }));
}

export function editorsWithinStructure(
  tabs: readonly SpaceEditorTab[],
  spaceId: string | undefined,
  snapshot: SpaceEditSnapshot,
  rootIds: readonly string[],
) {
  const nodeIds = new Set(contentNodeIdsWithin(rootIds, createStructureIndex(snapshot.structure)));
  return tabs.filter((tab) => tab.spaceId === spaceId && nodeIds.has(tab.nodeId));
}

export class EditorResourceProtectedError extends Error {
  constructor(readonly tab: SpaceEditorTab) {
    super(i18n.global.t("spaces.editor.tabs.protected"));
    this.name = "EditorResourceProtectedError";
  }
}

export function assertEditorResourcesRemovable(
  tabs: readonly SpaceEditorTab[],
  spaceId: string | undefined,
  snapshot: SpaceEditSnapshot,
  rootIds: readonly string[],
) {
  const tab = editorsWithinStructure(tabs, spaceId, snapshot, rootIds)[0];
  if (tab) throw new EditorResourceProtectedError(tab);
}
