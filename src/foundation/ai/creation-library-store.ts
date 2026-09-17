import { defineStore } from "pinia";

import {
  defaultCreationLibrary,
  loadCreationLibrary,
  resetCreationLibrary,
  saveCreationLibrary,
  type CreationLibrary,
} from "./integration-settings";
import { createIntegrationState } from "./integration-state";

export const useCreationLibraryStore = defineStore("creation-library", () => {
  const { value: library, ...state } = createIntegrationState<CreationLibrary>({
    initial: defaultCreationLibrary,
    load: loadCreationLibrary,
    save: saveCreationLibrary,
    reset: resetCreationLibrary,
  });

  return { library, ...state };
});
