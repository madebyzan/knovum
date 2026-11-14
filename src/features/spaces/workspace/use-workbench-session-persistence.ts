import { watch, type Ref } from "vue";

import type { Space } from "../model/space";
import { saveWorkbenchSession } from "./workbench-session";
import type { SpaceWorkbenchTab } from "./workbench-tab";

/** The live workbench state persistence follows, owned by the window session. */
export interface WorkbenchSessionSource {
  ready: Ref<boolean>;
  tabs: Ref<readonly SpaceWorkbenchTab[]>;
  activeTabId: Ref<string | undefined>;
  draftSpace: Ref<Space | undefined>;
}

/** Saves the strip whenever it changes once the workbench is ready, so a relaunch reopens it. */
export function useWorkbenchSessionPersistence(source: WorkbenchSessionSource) {
  watch(
    [source.ready, source.tabs, source.activeTabId, source.draftSpace],
    ([ready, tabs, activeTabId, draftSpace]) => {
      if (!ready) return;
      saveWorkbenchSession(tabs, activeTabId, draftSpace?.id);
    },
  );
}
