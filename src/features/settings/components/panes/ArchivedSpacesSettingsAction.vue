<script setup lang="ts">
import { Trash2 } from "@lucide/vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import { confirmAction } from "../../../../shared/ui/confirm-action";
import UiButton from "../../../../shared/ui/UiButton.vue";
import { useArchiveSettings } from "../../use-archive-settings";

const archive = useArchiveSettings();
const { t } = useI18n();

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
    toast.error(t("settings.general.archived.delete.failed"), { description: getErrorMessage(error) });
  }
}
</script>

<template>
  <UiButton v-if="archive.spaces.value.length" size="sm" variant="dangerSubtle" @click="permanentlyDeleteAll">
    <Trash2 class="size-3" />
    {{ t("settings.general.archived.deleteAll.label") }}
  </UiButton>
</template>
