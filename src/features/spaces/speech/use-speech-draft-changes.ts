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
  function acceptInitialization() {
    initialized.value = snapshot();
  }
  return { dirty, needsRevision, acceptRevision, acceptInitialization };
}
