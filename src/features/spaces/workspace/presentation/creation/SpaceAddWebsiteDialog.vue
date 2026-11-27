<script setup lang="ts">
import { useI18n } from "vue-i18n";

import UiButton from "../../../../../shared/ui/UiButton.vue";
import UiDialog from "../../../../../shared/ui/UiDialog.vue";
import UiInput from "../../../../../shared/ui/UiInput.vue";

const open = defineModel<boolean>("open", { required: true });
const website = defineModel<string>("website", { required: true });
defineProps<{ canSubmit: boolean }>();
const emit = defineEmits<{ submit: [] }>();
const { t } = useI18n();
</script>

<template>
  <UiDialog
    v-model:open="open"
    :title="t('spaces.importExport.website.add')"
    :description="t('spaces.importExport.website.description')"
  >
    <UiInput
      v-model="website"
      type="url"
      placeholder="example.com/article"
      @commit.prevent="emit('submit')"
    />
    <template #footer>
      <UiButton variant="ghost" @click="open = false">{{ t("common.cancel") }}</UiButton>
      <UiButton variant="primary" :disabled="!canSubmit" @click="emit('submit')">{{
        t("common.continue")
      }}</UiButton>
    </template>
  </UiDialog>
</template>
