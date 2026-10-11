import { isEqual } from "es-toolkit";
import { computed, shallowRef, type Ref } from "vue";

import type { VideoEditProject } from "./project";

const HISTORY_LIMIT = 50;

export function useVideoEditHistory(project: Ref<VideoEditProject | undefined>) {
  const undoStack = shallowRef<VideoEditProject[]>([]);
  const redoStack = shallowRef<VideoEditProject[]>([]);
  const savedSnapshot = shallowRef<VideoEditProject>();
  let lastMergeKey: string | undefined;
  let lastEditAt = 0;

  const canUndo = computed(() => undoStack.value.length > 0);
  const canRedo = computed(() => redoStack.value.length > 0);
  const dirty = computed(() => !!project.value && !isEqual(project.value, savedSnapshot.value));

  function reset(saved?: VideoEditProject) {
    undoStack.value = [];
    redoStack.value = [];
    savedSnapshot.value = saved;
    lastMergeKey = undefined;
  }

  function markSaved(snapshot: VideoEditProject) {
    savedSnapshot.value = snapshot;
  }

  function commit(next: VideoEditProject, mergeKey?: string) {
    const current = project.value;
    if (!current || isEqual(current, next)) return;
    const now = Date.now();
    if (!mergeKey || mergeKey !== lastMergeKey || now - lastEditAt > 600) {
      undoStack.value = [...undoStack.value, current].slice(-HISTORY_LIMIT);
    }
    project.value = next;
    redoStack.value = [];
    lastMergeKey = mergeKey;
    lastEditAt = now;
  }

  function undo() {
    const previous = undoStack.value.at(-1);
    if (!previous || !project.value) return;
    redoStack.value = [...redoStack.value, project.value];
    project.value = previous;
    undoStack.value = undoStack.value.slice(0, -1);
    lastMergeKey = undefined;
  }

  function redo() {
    const next = redoStack.value.at(-1);
    if (!next || !project.value) return;
    undoStack.value = [...undoStack.value, project.value];
    project.value = next;
    redoStack.value = redoStack.value.slice(0, -1);
    lastMergeKey = undefined;
  }

  return {
    canUndo,
    canRedo,
    dirty,
    reset,
    markSaved,
    commit,
    undo,
    redo,
    discard: () => {
      project.value = savedSnapshot.value;
      reset(savedSnapshot.value);
    },
  };
}
