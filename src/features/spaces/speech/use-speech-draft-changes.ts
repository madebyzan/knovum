import { computed, shallowRef } from "vue";

// Source initialization needs a revision for synthesis, but is not a user edit.
export function useSpeechDraftChanges(snapshot: () => string) {
  const persisted = shallowRef("");
  const initialized = shallowRef("");
  const dirty = computed(() => snapshot() !== initialized.value);
  const needsRevision = computed(() => snapshot() !== persisted.value);

  function acceptRevision() {
    persisted.value = snapshot();
    initialized.value = persisted.value;
  }
  /** Applies a source-derived change; it stays clean unless the person had already edited. */
  function initialize(change: () => void) {
    const wasDirty = dirty.value;
    change();
    if (!wasDirty) initialized.value = snapshot();
  }
  return { dirty, needsRevision, acceptRevision, initialize };
}
