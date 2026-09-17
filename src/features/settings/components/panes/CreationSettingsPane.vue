<script setup lang="ts">
import { ask } from "@tauri-apps/plugin-dialog";
import { storeToRefs } from "pinia";
import { onMounted, watch } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { useCreationLibraryStore } from "../../../../foundation/ai/creation-library-store";
import SavedCreationWorkflows from "../SavedCreationWorkflows.vue";
import SavedGenerationPresets from "../SavedGenerationPresets.vue";
import SettingsLoadFailure from "../SettingsLoadFailure.vue";

const { t } = useI18n();
const store = useCreationLibraryStore();
const { library, ready, loadError, saveError } = storeToRefs(store);

watch(saveError, (error) => {
  if (error) toast.error(`${t("settings.creation.saveFailed")}: ${String(error)}`);
});

async function resetSection() {
  const confirmed = await ask(t("settings.general.resetSectionConfirmation"), {
    title: t("settings.general.resetSection"),
    kind: "warning",
    okLabel: t("settings.general.resetSection"),
    cancelLabel: t("common.cancel"),
  });
  if (confirmed) await store.reset();
}

onMounted(store.ensureLoaded);
</script>

<template>
  <SettingsLoadFailure
    v-if="loadError"
    :message="String(loadError)"
    @retry="store.reload"
    @reset="resetSection"
  />
  <div v-else class="grid gap-5" :class="!ready && 'pointer-events-none opacity-60'">
    <p class="max-w-2xl text-body text-foreground-secondary">
      {{ t("settings.creation.description") }}
    </p>
    <SavedGenerationPresets v-model="library.generationPresets" />
    <SavedCreationWorkflows v-model="library.creationWorkflows" />
  </div>
</template>
