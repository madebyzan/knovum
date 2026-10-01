<script setup lang="ts">
import { Archive, RotateCcw, Trash2 } from "@lucide/vue";
import { inject, onMounted } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { errorMessage } from "../../../../foundation/errors/error-format";
import {
  archiveSettingsKey,
  type ArchivedSpace,
  type ArchiveSettingsContract,
} from "../../../../shared/contracts/archive-settings";
import { confirmAction } from "../../../../shared/ui/confirm-action";
import UiButton from "../../../../shared/ui/UiButton.vue";
import SettingsSection from "../SettingsSection.vue";

const archive = requireArchiveSettings();
const { t } = useI18n();

onMounted(() => archive.initialize(t("settings.general.inboxSpace")));

function requireArchiveSettings(): ArchiveSettingsContract {
  const contract = inject(archiveSettingsKey);
  if (!contract) throw new Error("Archive settings contract is required");
  return contract;
}

async function permanentlyDelete(space: ArchivedSpace) {
  const confirmed = await confirmAction({
    title: t("settings.general.archived.delete.title", { title: space.title }),
    description: t("settings.general.archived.delete.description"),
    confirmLabel: t("settings.general.archived.delete.label"),
    danger: true,
  });
  if (!confirmed) return;
  try {
    await archive.permanentlyDelete(space.id);
  } catch (error) {
    toast.error(t("settings.general.archived.delete.failed"), { description: errorMessage(error) });
  }
}
</script>

<template>
  <SettingsSection :title="t('settings.general.archived.title')">
    <article
      v-for="space in archive.spaces.value"
      :key="space.id"
      class="flex min-h-14 items-center gap-3 border-b border-stroke-divider px-4 last:border-b-0"
    >
      <Archive class="size-3.5 shrink-0 text-fg-secondary" :stroke-width="1.55" />
      <span class="min-w-0 flex-1 truncate text-label font-medium text-fg-primary">{{ space.title }}</span>
      <UiButton size="sm" variant="ghost" @click="archive.restore(space.id)">
        <RotateCcw class="size-3" />{{ t("settings.general.archived.restore") }}
      </UiButton>
      <UiButton size="sm" variant="ghost" @click="permanentlyDelete(space)">
        <Trash2 class="size-3" />{{ t("settings.general.archived.delete.label") }}
      </UiButton>
    </article>
    <div v-if="archive.spaces.value.length === 0" class="px-6 py-12 text-center">
      <Archive class="mx-auto size-4 text-fg-tertiary" :stroke-width="1.5" />
      <p class="mt-3 text-label text-fg-secondary">{{ t("settings.general.archived.empty") }}</p>
    </div>
  </SettingsSection>
</template>
