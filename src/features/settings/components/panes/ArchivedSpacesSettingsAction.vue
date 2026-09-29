<script setup lang="ts">
import { Trash2 } from "@lucide/vue";
import { inject } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import {
  archiveSettingsKey,
  type ArchiveSettingsContract,
} from "../../../../shared/contracts/archive-settings";
import { confirmAction } from "../../../../shared/ui/confirm-action";
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
  const confirmed = await confirmAction({
    title: t("settings.general.archived.deleteAll.title"),
    description: t("settings.general.archived.deleteAll.description", { count }),
    confirmLabel: t("settings.general.archived.deleteAll.label"),
    danger: true,
  });
  if (!confirmed) return;
  try {
    await archive.permanentlyDeleteAll();
  } catch (error) {
    toast.error(t("settings.general.archived.delete.failed"), { description: String(error) });
  }
}
</script>

<template>
  <UiButton v-if="archive.spaces.value.length" size="sm" variant="dangerSubtle" @click="permanentlyDeleteAll">
    <Trash2 class="size-3" />
    {{ t("settings.general.archived.deleteAll.label") }}
  </UiButton>
</template>
