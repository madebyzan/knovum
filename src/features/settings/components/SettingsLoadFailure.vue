<script setup lang="ts">
import { RotateCcw, Trash2 } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../shared/ui/UiButton.vue";
import UiSettingsSection from "../../../shared/ui/UiSettingsSection.vue";

withDefaults(defineProps<{ message: string; canReset?: boolean }>(), { canReset: true });
const emit = defineEmits<{ retry: []; reset: [] }>();
const { t } = useI18n();
</script>

<template>
  <UiSettingsSection :title="t('settings.general.load.failure.title')">
    <div class="px-4 py-3">
      <p class="text-body text-fg-danger">{{ message }}</p>
      <p class="mt-1 text-caption text-fg-secondary">
        {{ t("settings.general.load.failure.description") }}
      </p>
      <div class="mt-3 flex gap-2">
        <UiButton @click="emit('retry')"> <RotateCcw class="size-3.5" />{{ t("common.retry") }} </UiButton>
        <UiButton v-if="canReset" variant="danger" @click="emit('reset')">
          <Trash2 class="size-3.5" />{{ t("settings.general.reset.action") }}
        </UiButton>
      </div>
    </div>
  </UiSettingsSection>
</template>
