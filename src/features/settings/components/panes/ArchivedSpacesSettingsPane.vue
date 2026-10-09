<script setup lang="ts">
import { Archive, RotateCcw, Trash2 } from "@lucide/vue";
import { onMounted, shallowRef } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import type { ArchivedSpace } from "../../../../shared/contracts/archive-settings";
import { confirmAction } from "../../../../shared/ui/confirm-action";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiSettingsSection from "../../../../shared/ui/UiSettingsSection.vue";
import UiSpinner from "../../../../shared/ui/UiSpinner.vue";
import { useArchiveSettings } from "../../use-archive-settings";

const archive = useArchiveSettings();
const { t } = useI18n();
const initializing = shallowRef(false);
const loadError = shallowRef("");
const busySpaceIds = shallowRef<ReadonlySet<string>>(new Set());

onMounted(() => void initialize());

async function permanentlyDelete(space: ArchivedSpace) {
  const confirmed = await confirmAction({
    title: t("settings.general.archived.delete.title", { title: space.title }),
    description: t("settings.general.archived.delete.description"),
    confirmLabel: t("settings.general.archived.delete.label"),
    danger: true,
  });
  if (!confirmed) return;
  if (!beginSpaceOperation(space.id)) return;
  try {
    await archive.permanentlyDelete(space.id);
  } catch (error) {
    toast.error(t("settings.general.archived.delete.failed"), { description: getErrorMessage(error) });
  } finally {
    finishSpaceOperation(space.id);
  }
}

async function initialize() {
  if (initializing.value) return;
  initializing.value = true;
  loadError.value = "";
  try {
    await archive.initialize(t("settings.general.inboxSpace"));
  } catch (error) {
    loadError.value = getErrorMessage(error);
    toast.error(t("settings.general.archived.loadFailed"), { description: loadError.value });
  } finally {
    initializing.value = false;
  }
}

async function restore(id: string) {
  if (!beginSpaceOperation(id)) return;
  try {
    await archive.restore(id);
  } catch (error) {
    toast.error(t("settings.general.archived.restoreFailed"), { description: getErrorMessage(error) });
  } finally {
    finishSpaceOperation(id);
  }
}

function beginSpaceOperation(id: string) {
  if (busySpaceIds.value.has(id)) return false;
  busySpaceIds.value = new Set([...busySpaceIds.value, id]);
  return true;
}

function finishSpaceOperation(id: string) {
  const next = new Set(busySpaceIds.value);
  next.delete(id);
  busySpaceIds.value = next;
}
</script>

<template>
  <UiSettingsSection :title="t('settings.general.archived.title')">
    <div
      v-if="initializing"
      class="flex items-center gap-2 px-4 py-6 text-caption text-fg-secondary"
      role="status"
    >
      <UiSpinner size="sm" />{{ t("common.loading") }}
    </div>
    <div
      v-else-if="loadError"
      data-testid="archive-load-error"
      class="flex items-center gap-3 px-4 py-6 text-label text-fg-secondary"
      role="alert"
    >
      <span class="min-w-0 flex-1">{{ loadError }}</span>
      <UiButton data-testid="archive-retry" size="sm" @click="initialize">{{ t("common.retry") }}</UiButton>
    </div>
    <template v-else>
      <article
        v-for="space in archive.spaces.value"
        :key="space.id"
        class="flex min-h-14 items-center gap-3 border-b border-stroke-divider px-4 last:border-b-0"
      >
        <Archive class="size-3.5 shrink-0 text-fg-secondary" :stroke-width="1.55" />
        <span class="min-w-0 flex-1 truncate text-label font-medium text-fg-primary">{{ space.title }}</span>
        <UiButton
          data-testid="archive-restore-space"
          size="sm"
          variant="ghost"
          :disabled="busySpaceIds.has(space.id)"
          @click="restore(space.id)"
        >
          <RotateCcw class="size-3" />{{ t("settings.general.archived.restore") }}
        </UiButton>
        <UiButton
          size="sm"
          variant="ghost"
          :disabled="busySpaceIds.has(space.id)"
          @click="permanentlyDelete(space)"
        >
          <Trash2 class="size-3" />{{ t("settings.general.archived.delete.label") }}
        </UiButton>
      </article>
      <div v-if="archive.spaces.value.length === 0" class="px-6 py-12 text-center">
        <Archive class="mx-auto size-4 text-fg-tertiary" :stroke-width="1.5" />
        <p class="mt-3 text-label text-fg-secondary">{{ t("settings.general.archived.empty") }}</p>
      </div>
    </template>
  </UiSettingsSection>
</template>
