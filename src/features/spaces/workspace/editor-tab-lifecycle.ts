import { nextTick } from "vue";

import { prepareEditorSessionClose } from "./session-participants";
import type { SpaceSessionContext } from "./space-session-context";
import { removeSpaceWorkItems, type SpaceEditorTab } from "./work-items";

export async function prepareSpaceEditorClose(context: SpaceSessionContext, spaceIds: readonly string[]) {
  const tabs = context.editorTabs.value.filter((tab) => spaceIds.includes(tab.spaceId));
  return prepareEditorTabsClose(context, tabs);
}

export async function prepareEditorTabsClose(context: SpaceSessionContext, tabs: readonly SpaceEditorTab[]) {
  const finalize = await prepareEditorSessionClose({ tabIds: tabs.map(({ id }) => id) });
  return async () => {
    finalize();
    removeSpaceWorkItems(context, new Set(tabs.map(({ id }) => id)));
    await nextTick();
    for (const tab of tabs) context.editors.release(tab);
  };
}
