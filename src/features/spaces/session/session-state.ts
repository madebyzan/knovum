import { computed } from "vue";

import { createSpaceHistoryState } from "../history/history-state";
import { createSpaceMediaState } from "../media/media-state";
import { createWorkbenchNavigationState } from "../workspace/navigation-history";
import { spaceWorkbenchCapabilities } from "../workspace/space-surface";
import { createSpaceWorkbenchTabs } from "../workspace/workbench-tab-state";
import { createSpaceContentState } from "./content-state";
import { createSpaceUiState } from "./workbench-state";

export function createSpaceState() {
  const ui = createSpaceUiState();
  const content = createSpaceContentState();
  const workbench = createSpaceWorkbenchTabs(content);
  return {
    ...content,
    ...workbench,
    ...ui,
    workbench: computed(() =>
      spaceWorkbenchCapabilities(workbench.surface.value, Boolean(workbench.activeSpaceId.value)),
    ),
    ...createSpaceMediaState(),
    ...createSpaceHistoryState(),
    ...createWorkbenchNavigationState(),
  };
}

export type SpaceState = ReturnType<typeof createSpaceState>;

/**
 * Public store bindings exposed to the rest of the app.
 *
 * A handful of fields are deliberately kept out of the public surface
 * because they must only be mutated through a dedicated action rather than
 * assigned directly from a template/component (e.g. `thumbnailDirty` is
 * only toggled via `markCanvasOverviewDirty()`, navigation history only via
 * `recordWorkbenchNavigation()`). Everything else in `SpaceState` is
 * exposed automatically, so adding a new state field never requires a
 * second manual edit here to actually make it reachable from the store.
 */
const INTERNAL_ONLY_STATE_KEYS = [
  "thumbnailDirty",
  "workspaceDestination",
  "activeTab",
  "history",
  "navigationHistory",
  "navigationIndex",
] as const satisfies readonly (keyof SpaceState)[];

export function spaceSessionBindings(
  state: SpaceState,
): Omit<SpaceState, (typeof INTERNAL_ONLY_STATE_KEYS)[number]> {
  const bindings = { ...state };
  for (const key of INTERNAL_ONLY_STATE_KEYS) delete bindings[key];
  return bindings;
}
