<script setup lang="ts">
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiDialog from "../../../../shared/ui/UiDialog.vue";

defineProps<{ open: boolean; saving: boolean; canSave?: boolean; title: string; description: string }>();
const emit = defineEmits<{ save: []; discard: []; cancel: [] }>();
</script>

<template>
  <UiDialog :open="open" :title="title" :description="description" @update:open="!$event && emit('cancel')">
    <template #footer>
      <UiButton size="sm" :disabled="saving" @click="emit('cancel')">{{ $t("common.cancel") }}</UiButton>
      <UiButton size="sm" :disabled="saving" @click="emit('discard')">
        {{ $t("common.discardChanges") }}
      </UiButton>
      <UiButton size="sm" variant="primary" :disabled="saving || canSave === false" @click="emit('save')">
        {{ saving ? $t("common.saving") : $t("common.save") }}
      </UiButton>
    </template>
  </UiDialog>
</template>
