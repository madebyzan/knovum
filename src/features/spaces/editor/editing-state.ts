import { ref } from "vue";

export function createEditingState() {
  return { title: ref(""), revision: ref(0), persistedRevision: ref(0) };
}

export type EditingState = ReturnType<typeof createEditingState>;
