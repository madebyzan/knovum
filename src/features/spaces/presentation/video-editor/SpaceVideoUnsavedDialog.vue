<script setup lang="ts">
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiDialog from "../../../../shared/ui/UiDialog.vue";

defineProps<{ saving: boolean }>();
defineEmits<{ save: []; discard: [] }>();
const open = defineModel<boolean>("open", { required: true });
</script>

<template>
  <UiDialog
    v-model:open="open"
    :title="$t('spaces.media.unsavedTitle')"
    :description="$t('spaces.media.unsavedDescription')"
  >
    <template #footer>
      <UiButton size="sm" :disabled="saving" @click="open = false">{{ $t("common.cancel") }}</UiButton>
      <UiButton size="sm" :disabled="saving" @click="$emit('discard')">
        {{ $t("spaces.media.discardChanges") }}
      </UiButton>
      <UiButton size="sm" variant="primary" :disabled="saving" @click="$emit('save')">
        {{ saving ? $t("spaces.media.savingProject") : $t("common.save") }}
      </UiButton>
    </template>
  </UiDialog>
</template>
