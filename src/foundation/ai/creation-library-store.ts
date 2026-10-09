import { cloneDeep } from "es-toolkit";
import { defineStore } from "pinia";

import { isApplicationTransitioning } from "../lifecycle/application-transition";
import { loadCreationLibrary, resetCreationLibrary, saveCreationLibrary } from "./integration-settings";
import { defaultCreationLibrary, type CreationLibrary } from "./schema/creation-library";
import type { GenerationPreset } from "./schema/models";
import { useIntegrationSettingsState } from "./use-integration-settings-state";

function admit(operation: () => Promise<void>): Promise<void> {
  if (isApplicationTransitioning()) return Promise.reject(new Error("Creation presets are transitioning"));
  return operation();
}

export const useCreationLibraryStore = defineStore("creation-library", () => {
  const { value: library, ...state } = useIntegrationSettingsState<CreationLibrary>({
    initial: defaultCreationLibrary,
    load: loadCreationLibrary,
    save: saveCreationLibrary,
    reset: resetCreationLibrary,
  });

  function mutate(change: () => void): Promise<void> {
    return admit(() => {
      change();
      return state.flush();
    });
  }

  function addPreset(preset: GenerationPreset): Promise<void> {
    return mutate(() => {
      library.value.generationPresets.push(preset);
    });
  }

  function updatePreset(preset: GenerationPreset): Promise<void> {
    return mutate(() => {
      library.value.generationPresets = library.value.generationPresets.map((current) =>
        current.id === preset.id ? preset : current,
      );
    });
  }

  function renamePreset(presetId: string, name: string): Promise<void> {
    return mutate(() => {
      library.value.generationPresets.find((preset) => preset.id === presetId)!.name = name;
    });
  }

  function removePreset(presetId: string): Promise<void> {
    return mutate(() => {
      library.value.generationPresets = library.value.generationPresets.filter(
        (preset) => preset.id !== presetId,
      );
    });
  }

  function duplicatePreset(presetId: string, name: string): Promise<void> {
    return mutate(() => {
      const presets = library.value.generationPresets;
      const index = presets.findIndex((preset) => preset.id === presetId);
      presets.splice(index + 1, 0, { ...cloneDeep(presets[index]!), id: crypto.randomUUID(), name });
    });
  }

  return {
    library,
    ...state,
    addPreset,
    updatePreset,
    renamePreset,
    removePreset,
    duplicatePreset,
    reset: () => admit(state.reset),
  };
});
