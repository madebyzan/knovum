import { computed, ref, type Ref } from "vue";

/** Back/forward moves through workbench tab activations, not through Space switches alone. */
interface WorkbenchNavigationState {
  navigationHistory: Ref<string[]>;
  navigationIndex: Ref<number>;
}

export function recordWorkbenchNavigation(state: WorkbenchNavigationState, tabId: string) {
  const history = state.navigationHistory.value.slice(0, state.navigationIndex.value + 1);
  if (history.at(-1) === tabId) return;
  history.push(tabId);
  state.navigationHistory.value = history;
  state.navigationIndex.value = history.length - 1;
}

export function removeWorkbenchNavigation(state: WorkbenchNavigationState, removed: ReadonlySet<string>) {
  const currentIndex = state.navigationIndex.value;
  const current = state.navigationHistory.value[currentIndex];
  state.navigationHistory.value = state.navigationHistory.value.filter((tabId) => !removed.has(tabId));
  state.navigationIndex.value =
    current && !removed.has(current)
      ? state.navigationHistory.value.lastIndexOf(current)
      : Math.min(currentIndex, state.navigationHistory.value.length - 1);
}

export function createWorkbenchNavigationState() {
  const navigationHistory = ref<string[]>([]);
  const navigationIndex = ref(-1);

  return {
    navigationHistory,
    navigationIndex,
    canNavigateBack: computed(() => navigationIndex.value > 0),
    canNavigateForward: computed(() => navigationIndex.value < navigationHistory.value.length - 1),
  };
}
