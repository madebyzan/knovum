<script setup lang="ts">
import { useI18n } from "vue-i18n";

import { activeConfirmation, resolveConfirmation } from "./confirm-action";
import UiButton from "./UiButton.vue";
import UiDialog from "./UiDialog.vue";

const { t } = useI18n();

function handleOpenChange(open: boolean) {
  if (!open) resolveConfirmation(false);
}
</script>

<template>
  <UiDialog
    :open="Boolean(activeConfirmation)"
    :title="activeConfirmation?.title ?? ''"
    :closable="false"
    layer="confirmation"
    @update:open="handleOpenChange"
  >
    <p class="text-body leading-relaxed text-fg-secondary">{{ activeConfirmation?.description }}</p>
    <template #footer>
      <UiButton variant="ghost" @click="resolveConfirmation(false)">{{ t("common.cancel") }}</UiButton>
      <UiButton
        data-testid="confirm-action"
        :variant="activeConfirmation?.danger ? 'danger' : 'primary'"
        @click="resolveConfirmation(true)"
      >
        {{ activeConfirmation?.confirmLabel }}
      </UiButton>
    </template>
  </UiDialog>
</template>
