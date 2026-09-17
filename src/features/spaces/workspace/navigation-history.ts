import { computed, ref, type Ref } from "vue";

interface WorkspaceNavigationState {
  navigationHistory: Ref<string[]>;
  navigationIndex: Ref<number>;
}

export function recordWorkspaceNavigation(state: WorkspaceNavigationState, id: string) {
  const history = state.navigationHistory.value.slice(0, state.navigationIndex.value + 1);
  if (history.at(-1) === id) return;
  history.push(id);
  state.navigationHistory.value = history;
  state.navigationIndex.value = history.length - 1;
}

export function removeWorkspaceNavigation(state: WorkspaceNavigationState, id: string) {
  const currentIndex = state.navigationIndex.value;
  const current = state.navigationHistory.value[state.navigationIndex.value];
  state.navigationHistory.value = state.navigationHistory.value.filter((candidate) => candidate !== id);
  state.navigationIndex.value =
    current && current !== id
      ? state.navigationHistory.value.lastIndexOf(current)
      : Math.min(currentIndex, state.navigationHistory.value.length - 1);
}

export function createWorkspaceNavigationState() {
  const openSpaceIds = ref<string[]>([]);
  const closedSpaceIds = ref<string[]>([]);
  const navigationHistory = ref<string[]>([]);
  const navigationIndex = ref(-1);

  return {
    openSpaceIds,
    closedSpaceIds,
    navigationHistory,
    navigationIndex,
    canNavigateBack: computed(() => navigationIndex.value > 0),
    canNavigateForward: computed(() => navigationIndex.value < navigationHistory.value.length - 1),
  };
}
