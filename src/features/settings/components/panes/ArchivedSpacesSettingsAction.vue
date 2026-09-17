<script setup lang="ts">
import { Trash2 } from "@lucide/vue";
import { ask } from "@tauri-apps/plugin-dialog";
import { inject } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import {
  archiveSettingsKey,
  type ArchiveSettingsContract,
} from "../../../../shared/contracts/archive-settings";
import UiButton from "../../../../shared/ui/UiButton.vue";

const archive = requireArchiveSettings();
const { t } = useI18n();

function requireArchiveSettings(): ArchiveSettingsContract {
  const contract = inject(archiveSettingsKey);
  if (!contract) throw new Error("Archive settings contract is required");
  return contract;
}

async function permanentlyDeleteAll() {
  const count = archive.spaces.value.length;
  const confirmed = await ask(t("settings.general.deleteAllArchivedSpacesDescription", { count }), {
    title: t("settings.general.deleteAllArchivedSpacesTitle"),
    kind: "warning",
    okLabel: t("settings.general.deleteAllArchivedSpaces"),
    cancelLabel: t("common.cancel"),
  });
  if (!confirmed) return;
  try {
    await archive.permanentlyDeleteAll();
  } catch (error) {
    toast.error(t("spaces.navigation.deleteSpaceFailed"), { description: String(error) });
  }
}
</script>

<template>
  <UiButton v-if="archive.spaces.value.length" size="sm" variant="dangerSubtle" @click="permanentlyDeleteAll">
    <Trash2 class="size-(--knovum-icon-size-xs)" />
    {{ t("settings.general.deleteAllArchivedSpaces") }}
  </UiButton>
</template>
