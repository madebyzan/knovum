<script setup lang="ts">
import { computed, reactive, watch } from "vue";
import { useI18n } from "vue-i18n";

import AiExecutionTargetField from "../../../../foundation/ai/AiExecutionTargetField.vue";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiDialog from "../../../../shared/ui/UiDialog.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import UiTextarea from "../../../../shared/ui/UiTextarea.vue";
import type { MeetingContentPackageDraft } from "../../knowledge-derivations/workflows/meeting-content-package";

const open = defineModel<boolean>("open", { required: true });
const props = defineProps<{ sourceTitle: string }>();
const emit = defineEmits<{ submit: [draft: MeetingContentPackageDraft] }>();
const { t } = useI18n();
const form = reactive<MeetingContentPackageDraft>({ summaryTitle: "", summaryPurpose: "" });
const valid = computed(() =>
  Boolean(form.summaryTitle.trim() && form.summaryPurpose.trim() && form.textTarget),
);

watch(open, (isOpen) => {
  if (!isOpen) return;
  form.summaryTitle = `${props.sourceTitle} · ${t("spaces.creation.meeting.summary.title")}`;
  form.summaryPurpose = t("spaces.creation.meeting.summary.purpose");
});
</script>

<template>
  <UiDialog
    v-model:open="open"
    :title="t('spaces.creation.meeting.contentPackage')"
    :description="t('spaces.creation.meeting.videoDescription')"
  >
    <div class="space-y-4">
      <UiInput v-model="form.summaryTitle" :placeholder="t('common.title')" />
      <UiTextarea v-model="form.summaryPurpose" :rows="4" />
      <AiExecutionTargetField v-model="form.textTarget" purpose="knowledge" />
    </div>
    <template #footer>
      <UiButton variant="ghost" @click="open = false">{{ t("common.cancel") }}</UiButton>
      <UiButton variant="primary" :disabled="!valid" @click="emit('submit', { ...form })">
        {{ t("spaces.creation.meeting.create") }}
      </UiButton>
    </template>
  </UiDialog>
</template>
