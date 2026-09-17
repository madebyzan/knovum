import { computed, shallowReactive } from "vue";

import type { SpaceHistoryState } from "./stack";

export function createSpaceHistoryState() {
  const history = shallowReactive<SpaceHistoryState>({
    undoStack: [],
    redoStack: [],
    busy: false,
    pendingMutations: 0,
  });
  const available = computed(() => !history.busy && history.pendingMutations === 0);
  const canUndo = computed(() => history.undoStack.length > 0 && available.value);
  const canRedo = computed(() => history.redoStack.length > 0 && available.value);
  return { history, canUndo, canRedo };
}
