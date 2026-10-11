<script setup lang="ts">
import { reactive, watch } from "vue";
import { useI18n } from "vue-i18n";

import AiExecutionTargetField from "../../../../foundation/ai/AiExecutionTargetField.vue";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiDialog from "../../../../shared/ui/UiDialog.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import UiTextarea from "../../../../shared/ui/UiTextarea.vue";
import type { VideoDraft } from "../video/generation";

const open = defineModel<boolean>("open", { required: true });
const props = defineProps<{ sourceTitle: string }>();
const emit = defineEmits<{ submit: [draft: VideoDraft] }>();
const { t } = useI18n();
const DURATION_OPTIONS = [
  { value: "5", label: "5s", seconds: 5 },
  { value: "10", label: "10s", seconds: 10 },
  { value: "15", label: "15s", seconds: 15 },
] as const;
const form = reactive<VideoDraft>({
  title: "",
  prompt: "",
  aspectRatio: "16:9",
  durationSeconds: 5,
});
watch(open, (isOpen) => {
  if (isOpen) {
    form.title = `${props.sourceTitle} · Video`;
    form.prompt = "";
  }
});

function selectDuration(value: string) {
  const option = DURATION_OPTIONS.find((duration) => duration.value === value);
  if (option) form.durationSeconds = option.seconds;
}
</script>
<template>
  <UiDialog
    v-model:open="open"
    :title="t('spaces.creation.video.generation.title')"
    :description="t('spaces.creation.video.generation.description')"
  >
    <div class="space-y-4">
      <UiInput v-model="form.title" :placeholder="t('spaces.creation.video.generation.name')" /><UiTextarea
        v-model="form.prompt"
        :rows="5"
        :placeholder="t('spaces.creation.video.generation.prompt')"
      />
      <div class="grid grid-cols-2 gap-3">
        <UiSelect
          v-model="form.aspectRatio"
          :label="t('spaces.creation.image.aspectRatio')"
          :options="[
            { value: '16:9', label: '16:9' },
            { value: '9:16', label: '9:16' },
            { value: '1:1', label: '1:1' },
          ]"
        /><UiSelect
          :model-value="String(form.durationSeconds)"
          :label="t('spaces.creation.video.generation.duration')"
          :options="DURATION_OPTIONS"
          @update:model-value="selectDuration"
        />
      </div>
      <AiExecutionTargetField v-model="form.target" purpose="video-generation" />
    </div>
    <template #footer
      ><UiButton variant="ghost" @click="open = false">{{ t("common.cancel") }}</UiButton
      ><UiButton
        variant="primary"
        :disabled="!form.title.trim() || !form.prompt.trim() || !form.target"
        @click="emit('submit', { ...form })"
        >{{ t("spaces.creation.video.generation.create") }}</UiButton
      ></template
    >
  </UiDialog>
</template>
