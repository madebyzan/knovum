import { nextTick } from "vue";

import type { SpaceSessionContext } from "./space-session-context";
import type { SpaceEditorTab } from "./workbench-tab";

export function editorTabsForSpace(context: SpaceSessionContext, spaceId: string) {
  return context.editorTabs.value.filter((tab) => tab.spaceId === spaceId);
}

/** Editor views unmount before their vendor instances are released, never the other way round. */
export async function releaseEditorTabs(context: SpaceSessionContext, tabs: readonly SpaceEditorTab[]) {
  if (!tabs.length) return;
  await nextTick();
  for (const tab of tabs) context.editors.release(tab);
}
