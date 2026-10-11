<script setup lang="ts">
import { storeToRefs } from "pinia";
import { onMounted } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { useCreationLibraryStore } from "../../../foundation/ai/creation-library-store";
import { getErrorMessage } from "../../../foundation/errors/error-format";
import { confirmAction } from "../../../shared/ui/confirm-action";
import SettingsLoadFailure from "../components/SettingsLoadFailure.vue";
import { useSettingsLeaveDecision } from "../navigation/use-settings-leave-decision";
import type { GenerationActionOption } from "./action-option";
import FalImageLayerSettings from "./components/FalImageLayerSettings.vue";
import SavedGenerationPresets from "./components/SavedGenerationPresets.vue";

const props = defineProps<{ loadGenerationActions: () => Promise<GenerationActionOption[]> }>();
const { t } = useI18n();
const store = useCreationLibraryStore();
const { library, ready, loadError } = storeToRefs(store);

async function updatePresets(operation: () => Promise<void>) {
  try {
    await operation();
  } catch (error) {
    toast.error(`${t("settings.creation.saveFailed")}: ${getErrorMessage(error)}`);
  }
}

async function resetSection() {
  const confirmed = await confirmAction({
    title: t("settings.general.reset.action"),
    description: t("settings.general.reset.confirmation"),
    confirmLabel: t("settings.general.reset.action"),
    danger: true,
  });
  if (confirmed) await updatePresets(store.reset);
}

useSettingsLeaveDecision({
  async prepare() {
    try {
      await store.flush();
      return true;
    } catch (error) {
      toast.error(`${t("settings.creation.saveFailed")}: ${getErrorMessage(error)}`);
      return false;
    }
  },
});

onMounted(store.ensureLoaded);
</script>

<template>
  <SettingsLoadFailure
    v-if="loadError"
    :message="getErrorMessage(loadError)"
    @retry="store.reload"
    @reset="resetSection"
  />
  <div v-else class="grid gap-5" :class="!ready && 'pointer-events-none opacity-60'">
    <p class="max-w-2xl text-body text-fg-secondary">
      {{ t("settings.creation.description") }}
    </p>
    <SavedGenerationPresets
      :presets="library.generationPresets"
      :load-generation-actions="props.loadGenerationActions"
      :save-preset="store.updatePreset"
      @remove="(id) => updatePresets(() => store.removePreset(id))"
      @duplicate="(id, name) => updatePresets(() => store.duplicatePreset(id, name))"
      @rename="(id, name) => updatePresets(() => store.renamePreset(id, name))"
    />
    <FalImageLayerSettings />
  </div>
</template>
