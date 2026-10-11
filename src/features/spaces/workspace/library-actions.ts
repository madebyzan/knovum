import { nextTick } from "vue";

import type { SpaceLibraryScope } from "../model/space";
import type { SpaceState } from "../session/session-state";
import type { SpaceSessionContext } from "./space-session-context";
import { transitionActiveSpaceSession } from "./space-transition";

export async function showSpaceLibrary(
  state: SpaceState,
  context: SpaceSessionContext,
  scope: SpaceLibraryScope,
) {
  if (state.surface.value.kind === "library") {
    state.libraryScope.value = scope;
    return;
  }
  const space = state.activeSpace.value;
  if (!space) throw new Error("Space requires an active space");
  await transitionActiveSpaceSession(context, async () => {
    state.libraryScope.value = scope;
    state.surface.value = { kind: "library" };
    await nextTick();
  });
}
