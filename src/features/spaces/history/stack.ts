import { beginSpaceMutation } from "./mutation-barrier";

export interface SpaceHistoryEntry {
  undo: () => Promise<void>;
  redo: () => Promise<void>;
}

export interface SpaceHistoryState {
  undoStack: readonly SpaceHistoryEntry[];
  redoStack: readonly SpaceHistoryEntry[];
  busy: boolean;
  pendingMutations: number;
}

const MAX_HISTORY_STACK = 100;

// The stacks are held in a shallowReactive object, so every mutation replaces the array instead of
// editing it in place; otherwise `canUndo`/`canRedo` never observe a change.
export function recordSpaceHistory(state: SpaceHistoryState, entry: SpaceHistoryEntry): void {
  state.undoStack = [...state.undoStack, entry].slice(-MAX_HISTORY_STACK);
  state.redoStack = [];
}

export async function undoSpaceHistory(state: SpaceHistoryState): Promise<void> {
  if (state.busy || state.pendingMutations > 0) return;
  const entry = state.undoStack.at(-1);
  if (!entry) return;
  const releaseMutation = beginSpaceMutation(state);
  state.undoStack = state.undoStack.slice(0, -1);
  state.busy = true;
  try {
    await entry.undo();
    state.redoStack = [...state.redoStack, entry];
  } catch (error) {
    state.undoStack = [...state.undoStack, entry];
    throw error;
  } finally {
    state.busy = false;
    releaseMutation();
  }
}

export async function redoSpaceHistory(state: SpaceHistoryState): Promise<void> {
  if (state.busy || state.pendingMutations > 0) return;
  const entry = state.redoStack.at(-1);
  if (!entry) return;
  const releaseMutation = beginSpaceMutation(state);
  state.redoStack = state.redoStack.slice(0, -1);
  state.busy = true;
  try {
    await entry.redo();
    state.undoStack = [...state.undoStack, entry];
  } catch (error) {
    state.redoStack = [...state.redoStack, entry];
    throw error;
  } finally {
    state.busy = false;
    releaseMutation();
  }
}

export function combineRecentSpaceHistory(state: SpaceHistoryState, startIndex: number): void {
  const entries = state.undoStack.slice(startIndex);
  if (entries.length <= 1) return;
  state.undoStack = [
    ...state.undoStack.slice(0, startIndex),
    {
      undo: async () => {
        for (const entry of entries.toReversed()) {
          // oxlint-disable-next-line eslint/no-await-in-loop -- Compound undo must complete strictly in reverse mutation order.
          await entry.undo();
        }
      },
      redo: async () => {
        for (const entry of entries) {
          // oxlint-disable-next-line eslint/no-await-in-loop -- Compound redo must complete strictly in original mutation order.
          await entry.redo();
        }
      },
    },
  ];
}

export function clearSpaceHistory(state: SpaceHistoryState): void {
  state.undoStack = [];
  state.redoStack = [];
}
