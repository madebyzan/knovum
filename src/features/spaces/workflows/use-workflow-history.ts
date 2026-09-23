import { shallowRef, type Ref } from "vue";

import type { WorkflowDraft } from "./workflow-client";

export function useWorkflowHistory(draft: Ref<WorkflowDraft | undefined>) {
  const undoStack = shallowRef<string[]>([]);
  const redoStack = shallowRef<string[]>([]);
  let currentSnapshot = "";
  let savedSnapshot = "";
  let restoring = false;

  function reset(value: WorkflowDraft) {
    currentSnapshot = JSON.stringify(value.definition);
    savedSnapshot = currentSnapshot;
    undoStack.value = [];
    redoStack.value = [];
  }

  function markSaved() {
    if (draft.value) {
      currentSnapshot = JSON.stringify(draft.value.definition);
      savedSnapshot = currentSnapshot;
    }
  }

  function track(definition: WorkflowDraft["definition"]) {
    if (restoring) return false;
    const next = JSON.stringify(definition);
    if (currentSnapshot && next !== currentSnapshot) {
      undoStack.value = [...undoStack.value.slice(-49), currentSnapshot];
      redoStack.value = [];
    }
    const changed = next !== currentSnapshot;
    currentSnapshot = next;
    return changed;
  }

  function restore(stack: "undo" | "redo") {
    if (!draft.value) return;
    const source = stack === "undo" ? undoStack : redoStack;
    const target = stack === "undo" ? redoStack : undoStack;
    const snapshot = source.value.at(-1);
    if (!snapshot) return;
    target.value = [...target.value, JSON.stringify(draft.value.definition)];
    source.value = source.value.slice(0, -1);
    restoring = true;
    draft.value.definition = JSON.parse(snapshot) as WorkflowDraft["definition"];
    currentSnapshot = snapshot;
    restoring = false;
  }

  const needsSave = (definition: WorkflowDraft["definition"]) => JSON.stringify(definition) !== savedSnapshot;

  return { undoStack, redoStack, markSaved, needsSave, reset, restore, track };
}
