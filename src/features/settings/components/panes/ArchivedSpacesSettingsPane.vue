<script setup lang="ts">
import { Archive, RotateCcw, Trash2 } from "@lucide/vue";
import { ask } from "@tauri-apps/plugin-dialog";
import { inject, onMounted } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import {
  archiveSettingsKey,
  type ArchivedSpaceSummary,
  type ArchiveSettingsContract,
} from "../../../../shared/contracts/archive-settings";
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

async function permanentlyDelete(space: ArchivedSpaceSummary) {
  const confirmed = await ask(t("settings.general.archived.delete.description"), {
    title: t("settings.general.archived.delete.title", { title: space.title }),
    kind: "warning",
    okLabel: t("settings.general.archived.delete.label"),
    cancelLabel: t("common.cancel"),
  });
  if (!confirmed) return;
  try {
    await archive.permanentlyDelete(space.id);
  } catch (error) {
    toast.error(t("settings.general.archived.delete.failed"), { description: String(error) });
  }
}
</script>

<template>
  <SettingsSection :title="t('settings.general.archived.title')">
    <article
      v-for="space in archive.spaces.value"
      :key="space.id"
      class="flex min-h-14 items-center gap-3 border-b border-divider px-4 last:border-b-0"
    >
      <Archive class="size-(--knovum-icon-size-sm) shrink-0 text-foreground-secondary" :stroke-width="1.55" />
      <span class="min-w-0 flex-1 truncate text-label font-medium text-foreground">{{ space.title }}</span>
      <UiButton size="sm" variant="ghost" @click="archive.restore(space.id)">
        <RotateCcw class="size-(--knovum-icon-size-xs)" />{{ t("settings.general.archived.restore") }}
      </UiButton>
      <UiButton size="sm" variant="ghost" @click="permanentlyDelete(space)">
        <Trash2 class="size-(--knovum-icon-size-xs)" />{{ t("settings.general.archived.delete.label") }}
      </UiButton>
    </article>
    <div v-if="archive.spaces.value.length === 0" class="px-6 py-12 text-center">
      <Archive class="mx-auto size-(--knovum-icon-size-md) text-foreground-tertiary" :stroke-width="1.5" />
      <p class="mt-3 text-label text-foreground-secondary">{{ t("settings.general.archived.empty") }}</p>
    </div>
  </SettingsSection>
</template>
