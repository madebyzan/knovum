import { defineStore } from "pinia";

import { loadCreationLibrary, resetCreationLibrary, saveCreationLibrary } from "./integration-settings";
import { createIntegrationState } from "./integration-state";
import { defaultCreationLibrary, type CreationLibrary } from "./schema/creation-library";

export const useCreationLibraryStore = defineStore("creation-library", () => {
  const { value: library, ...state } = createIntegrationState<CreationLibrary>({
    initial: defaultCreationLibrary,
    load: loadCreationLibrary,
    save: saveCreationLibrary,
    reset: resetCreationLibrary,
  });

  return { library, ...state };
});
