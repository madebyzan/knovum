import { describe, expect, it } from "vitest";

import {
  createWorkspaceNavigationState,
  recordWorkspaceNavigation,
  removeWorkspaceNavigation,
} from "../navigation-history";

describe("workspace tab navigation", () => {
  it("drops forward history when a new space is opened after going back", () => {
    const state = createWorkspaceNavigationState();
    recordWorkspaceNavigation(state, "a");
    recordWorkspaceNavigation(state, "b");
    recordWorkspaceNavigation(state, "c");
    state.navigationIndex.value = 1;

    recordWorkspaceNavigation(state, "d");

    expect(state.navigationHistory.value).toEqual(["a", "b", "d"]);
    expect(state.navigationIndex.value).toBe(2);
  });

  it("selects the adjacent history entry when the current tab closes", () => {
    const state = createWorkspaceNavigationState();
    state.navigationHistory.value = ["a", "b", "c"];
    state.navigationIndex.value = 2;

    removeWorkspaceNavigation(state, "c");

    expect(state.navigationHistory.value).toEqual(["a", "b"]);
    expect(state.navigationIndex.value).toBe(1);
    expect(state.canNavigateBack.value).toBe(true);
    expect(state.canNavigateForward.value).toBe(false);
  });

  it("preserves the current entry when a background tab closes", () => {
    const state = createWorkspaceNavigationState();
    state.navigationHistory.value = ["a", "b", "c"];
    state.navigationIndex.value = 2;

    removeWorkspaceNavigation(state, "b");

    expect(state.navigationHistory.value).toEqual(["a", "c"]);
    expect(state.navigationIndex.value).toBe(1);
  });
});
