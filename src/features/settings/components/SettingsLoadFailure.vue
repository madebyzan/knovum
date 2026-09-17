<script setup lang="ts">
import { RotateCcw, Trash2 } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../shared/ui/UiButton.vue";
import SettingsSection from "./SettingsSection.vue";

withDefaults(defineProps<{ message: string; canReset?: boolean }>(), { canReset: true });
const emit = defineEmits<{ retry: []; reset: [] }>();
const { t } = useI18n();
</script>

<template>
  <SettingsSection :title="t('settings.general.settingsLoadFailed')">
    <div class="px-4 py-3">
      <p class="text-body text-danger">{{ message }}</p>
      <p class="mt-1 text-caption text-foreground-secondary">
        {{ t("settings.general.settingsLoadFailedDescription") }}
      </p>
      <div class="mt-3 flex gap-2">
        <UiButton @click="emit('retry')">
          <RotateCcw class="size-(--knovum-icon-size-sm)" />{{ t("common.retry") }}
        </UiButton>
        <UiButton v-if="canReset" variant="danger" @click="emit('reset')">
          <Trash2 class="size-(--knovum-icon-size-sm)" />{{ t("settings.general.resetSection") }}
        </UiButton>
      </div>
    </div>
  </SettingsSection>
</template>
