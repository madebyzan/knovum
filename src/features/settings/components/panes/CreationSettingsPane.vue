<script setup lang="ts">
import type { GenerationActionOption } from "../../generation-action-option";

const props = defineProps<{ loadGenerationActions: () => Promise<GenerationActionOption[]> }>();
import { ask } from "@tauri-apps/plugin-dialog";
import { storeToRefs } from "pinia";
import { onMounted, watch } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { useCreationLibraryStore } from "../../../../foundation/ai/creation-library-store";
import SavedGenerationPresets from "../SavedGenerationPresets.vue";
import SettingsLoadFailure from "../SettingsLoadFailure.vue";

const { t } = useI18n();
const store = useCreationLibraryStore();
const { library, ready, loadError, saveError } = storeToRefs(store);

watch(saveError, (error) => {
  if (error) toast.error(`${t("settings.creation.saveFailed")}: ${String(error)}`);
});

async function resetSection() {
  const confirmed = await ask(t("settings.general.reset.confirmation"), {
    title: t("settings.general.reset.action"),
    kind: "warning",
    okLabel: t("settings.general.reset.action"),
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
    <p class="max-w-2xl text-body text-fg-secondary">
      {{ t("settings.creation.description") }}
    </p>
    <SavedGenerationPresets
      v-model="library.generationPresets"
      :load-generation-actions="props.loadGenerationActions"
    />
  </div>
</template>
