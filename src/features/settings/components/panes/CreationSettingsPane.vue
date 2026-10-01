<script setup lang="ts">
import { getErrorMessage } from "../../../../foundation/errors/error-format";
import type { GenerationActionOption } from "../../generation-action-option";

const props = defineProps<{ loadGenerationActions: () => Promise<GenerationActionOption[]> }>();
import { storeToRefs } from "pinia";
import { onMounted, watch } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { useCreationLibraryStore } from "../../../../foundation/ai/creation-library-store";
import { confirmAction } from "../../../../shared/ui/confirm-action";
import FalImageLayerSettings from "../FalImageLayerSettings.vue";
import SavedGenerationPresets from "../SavedGenerationPresets.vue";
import SettingsLoadFailure from "../SettingsLoadFailure.vue";

const { t } = useI18n();
const store = useCreationLibraryStore();
const { library, ready, loadError, saveError } = storeToRefs(store);

watch(saveError, (error) => {
  if (error) toast.error(`${t("settings.creation.saveFailed")}: ${getErrorMessage(error)}`);
});

async function resetSection() {
  const confirmed = await confirmAction({
    title: t("settings.general.reset.action"),
    description: t("settings.general.reset.confirmation"),
    confirmLabel: t("settings.general.reset.action"),
    danger: true,
  });
  if (confirmed) await store.reset();
}

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
      v-model="library.generationPresets"
      :load-generation-actions="props.loadGenerationActions"
    />
    <FalImageLayerSettings />
  </div>
</template>
