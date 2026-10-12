import { describe, expect, it } from "vitest";

import {
  createWorkbenchNavigationState,
  recordWorkbenchNavigation,
  removeWorkbenchNavigation,
} from "../navigation-history";

describe("workbench tab navigation", () => {
  it("drops forward history when a new tab is activated after going back", () => {
    const state = createWorkbenchNavigationState();
    recordWorkbenchNavigation(state, "a");
    recordWorkbenchNavigation(state, "b");
    recordWorkbenchNavigation(state, "c");
    state.navigationIndex.value = 1;

    recordWorkbenchNavigation(state, "d");

    expect(state.navigationHistory.value).toEqual(["a", "b", "d"]);
    expect(state.navigationIndex.value).toBe(2);
  });

  it("selects the adjacent history entry when the current tab closes", () => {
    const state = createWorkbenchNavigationState();
    state.navigationHistory.value = ["a", "b", "c"];
    state.navigationIndex.value = 2;

    removeWorkbenchNavigation(state, new Set(["c"]));

    expect(state.navigationHistory.value).toEqual(["a", "b"]);
    expect(state.navigationIndex.value).toBe(1);
    expect(state.canNavigateBack.value).toBe(true);
    expect(state.canNavigateForward.value).toBe(false);
  });

  it("preserves the current entry when background tabs close", () => {
    const state = createWorkbenchNavigationState();
    state.navigationHistory.value = ["a", "b", "c", "d"];
    state.navigationIndex.value = 3;

    removeWorkbenchNavigation(state, new Set(["b", "c"]));

    expect(state.navigationHistory.value).toEqual(["a", "d"]);
    expect(state.navigationIndex.value).toBe(1);
  });
});
