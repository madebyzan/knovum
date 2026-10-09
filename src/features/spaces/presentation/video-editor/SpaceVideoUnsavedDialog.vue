<script setup lang="ts">
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiDialog from "../../../../shared/ui/UiDialog.vue";

defineProps<{ saving: boolean }>();
const emit = defineEmits<{ save: []; discard: []; cancel: [] }>();
const open = defineModel<boolean>("open", { required: true });
</script>

<template>
  <UiDialog
    :open="open"
    :allowed-command-ids="['video-edit.close']"
    @update:open="!$event && emit('cancel')"
    :title="$t('spaces.media.videoEditor.unsaved.title')"
    :description="$t('spaces.media.videoEditor.unsaved.description')"
  >
    <template #footer>
      <UiButton size="sm" :disabled="saving" @click="emit('cancel')">{{ $t("common.cancel") }}</UiButton>
      <UiButton size="sm" :disabled="saving" @click="$emit('discard')">
        {{ $t("common.discardChanges") }}
      </UiButton>
      <UiButton size="sm" variant="primary" :disabled="saving" @click="$emit('save')">
        {{ saving ? $t("common.saving") : $t("common.save") }}
      </UiButton>
    </template>
  </UiDialog>
</template>
