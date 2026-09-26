<script setup lang="ts">
import { computed, reactive, watch } from "vue";
import { useI18n } from "vue-i18n";

import AiExecutionTargetField from "../../../../foundation/ai/AiExecutionTargetField.vue";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiDialog from "../../../../shared/ui/UiDialog.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import UiTextarea from "../../../../shared/ui/UiTextarea.vue";
import type { PresentationGenerationDraft } from "../../knowledge-derivations/workflows/presentation-production";

const open = defineModel<boolean>("open", { required: true });
const props = defineProps<{ sourceTitle: string }>();
const emit = defineEmits<{ submit: [draft: PresentationGenerationDraft] }>();
const { t } = useI18n();
const form = reactive<PresentationGenerationDraft>({ title: "", purpose: "" });
const valid = computed(() => Boolean(form.title.trim() && form.purpose.trim() && form.textTarget));

watch(open, (isOpen) => {
  if (!isOpen) return;
  form.title = `${props.sourceTitle} · ${t("spaces.creation.presentation.outline.title")}`;
  form.purpose = t("spaces.creation.presentation.outline.purpose");
});
</script>

<template>
  <UiDialog
    v-model:open="open"
    :title="t('spaces.creation.presentation.create')"
    :description="t('spaces.creation.presentation.description')"
  >
    <div class="space-y-4">
      <UiInput v-model="form.title" :placeholder="t('common.title')" />
      <UiTextarea v-model="form.purpose" :rows="4" />
      <AiExecutionTargetField v-model="form.textTarget" purpose="knowledge" />
    </div>
    <template #footer>
      <UiButton variant="ghost" @click="open = false">{{ t("common.cancel") }}</UiButton>
      <UiButton variant="primary" :disabled="!valid" @click="emit('submit', { ...form })">
        {{ t("spaces.creation.presentation.create") }}
      </UiButton>
    </template>
  </UiDialog>
</template>
