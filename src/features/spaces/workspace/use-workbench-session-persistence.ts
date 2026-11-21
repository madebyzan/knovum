import { watch, type Ref } from "vue";

import type { Space } from "../model/space";
import type { RightSidebarPrimaryView, RightSidebarView } from "../session/workbench-state";
import {
  captureSavedSpaceView,
  serializeSpaceViewSession,
  type SavedSpaceViewSession,
  type SpaceViewRefs,
  type SpaceViewSession,
} from "./space-view-session";
import { saveWorkbenchSession, type SaveWorkbenchSessionInput } from "./workbench-session";
import type { SpaceWorkbenchTab } from "./workbench-tab";

/** The live workbench state persistence follows, owned by the window session. */
export interface WorkbenchSessionSource extends Pick<
  SpaceViewRefs,
  "groupScopePath" | "activeSceneId" | "outlineExpandedIds"
> {
  ready: Ref<boolean>;
  tabs: Ref<readonly SpaceWorkbenchTab[]>;
  activeTabId: Ref<string | undefined>;
  activeSpaceId: Ref<string | undefined>;
  draftSpace: Ref<Space | undefined>;
  rightSidebarView: Ref<RightSidebarView>;
  sourceReturnView: Ref<RightSidebarPrimaryView>;
  runtime: {
    retained: ReadonlyMap<string, { view: SpaceViewSession }>;
  };
}

/** Saves the workbench session whenever any tracked navigation or Space view state changes once ready. */
export function useWorkbenchSessionPersistence(source: WorkbenchSessionSource) {
  watch(
    () => collectSessionInput(source),
    (snapshot) => {
      if (snapshot) saveWorkbenchSession(snapshot);
    },
  );
}

function collectSessionInput(source: WorkbenchSessionSource): SaveWorkbenchSessionInput | undefined {
  if (!source.ready.value) return undefined;
  const spaceViews = new Map<string, SavedSpaceViewSession>();
  for (const [spaceId, retained] of source.runtime.retained) {
    const serialized = serializeSpaceViewSession(retained.view);
    if (serialized) spaceViews.set(spaceId, serialized);
  }
  if (source.activeSpaceId.value) {
    const active = captureSavedSpaceView(source);
    if (active) spaceViews.set(source.activeSpaceId.value, active);
  }
  return {
    tabs: source.tabs.value,
    activeTabId: source.activeTabId.value,
    draftSpaceId: source.draftSpace.value?.id,
    rightSidebarView: source.rightSidebarView.value,
    sourceReturnView: source.sourceReturnView.value,
    spaceViews,
  };
}
