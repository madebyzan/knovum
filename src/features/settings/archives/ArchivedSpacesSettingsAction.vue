<script setup lang="ts">
import { Trash2 } from "@lucide/vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { confirmAction } from "../../../shared/ui/confirm-action";
import UiButton from "../../../shared/ui/UiButton.vue";
import UiSettingsHeaderActions from "../../../shared/ui/UiSettingsHeaderActions.vue";
import { useArchiveSettings } from "./use-archive-settings";

const archive = useArchiveSettings();
const { t } = useI18n();

async function permanentlyDeleteAll() {
  const count = archive.spaces.value.length;
  const confirmed = await confirmAction({
    title: t("settings.archivedSpaces.deleteAll.title"),
    description: t("settings.archivedSpaces.deleteAll.description", { count }),
    confirmLabel: t("settings.archivedSpaces.deleteAll.action"),
    danger: true,
  });
  if (!confirmed) return;
  try {
    await archive.permanentlyDeleteAll();
  } catch (error) {
    toast.error(t("settings.archivedSpaces.delete.failed"), { description: getErrorMessage(error) });
  }
}
</script>

<template>
  <UiSettingsHeaderActions v-if="archive.spaces.value.length">
    <UiButton size="sm" variant="dangerSubtle" @click="permanentlyDeleteAll">
      <Trash2 class="size-3" />
      {{ t("settings.archivedSpaces.deleteAll.action") }}
    </UiButton>
  </UiSettingsHeaderActions>
</template>
