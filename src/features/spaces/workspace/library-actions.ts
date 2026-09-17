import { scheduleCanvasThumbnail } from "../canvas/thumbnail-scheduler";
import type { SpaceLibraryScope } from "../model/space";
import type { SpaceState } from "../session/session-state";

export function showSpaceLibrary(state: SpaceState, scope: SpaceLibraryScope) {
  if (state.workspaceMode.value === "library") {
    state.libraryScope.value = scope;
    return;
  }
  const space = state.activeSpace.value;
  if (!space) throw new Error("Space requires an active space");
  state.libraryScope.value = scope;
  state.workspaceMode.value = "library";
  scheduleCanvasThumbnail(state);
}
