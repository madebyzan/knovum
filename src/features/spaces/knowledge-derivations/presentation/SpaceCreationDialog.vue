<script setup lang="ts">
import { computed, reactive, watch } from "vue";
import { useI18n } from "vue-i18n";

import AiExecutionTargetField from "../../../../foundation/ai/AiExecutionTargetField.vue";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiDialog from "../../../../shared/ui/UiDialog.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import UiTextarea from "../../../../shared/ui/UiTextarea.vue";
import type { CreationSource } from "../catalog/creation-source";
import {
  CREATION_OUTPUTS,
  creationTargetPurpose,
  imageAspectRatioSchema,
  videoAspectRatioSchema,
  videoDurations,
  type CreationDraft,
  type CreationOutput,
} from "../creation";

const open = defineModel<boolean>("open", { required: true });
const props = defineProps<{ output: CreationOutput; sources: readonly CreationSource[] }>();
const emit = defineEmits<{ submit: [draft: CreationDraft] }>();
const { t } = useI18n();

const form = reactive<CreationDraft>({
  output: props.output,
  title: "",
  requirement: "",
  imageAspectRatio: "16:9",
  videoAspectRatio: "16:9",
  durationSeconds: 5,
});
const valid = computed(() => Boolean(form.title.trim() && form.requirement.trim() && form.target));
const outputs = computed(() =>
  CREATION_OUTPUTS.map((value) => ({ value, label: t(`spaces.creation.composer.output.${value}`) })),
);
const imageRatios = imageAspectRatioSchema.options.map((value) => ({ value, label: value }));
const videoRatios = videoAspectRatioSchema.options.map((value) => ({ value, label: value }));
const durations = videoDurations.map((seconds) => ({ value: String(seconds), label: `${seconds}s` }));

watch(
  [open, () => props.output],
  ([isOpen, output]) => {
    if (!isOpen) return;
    form.output = output;
    form.requirement = "";
    form.title = t(`spaces.creation.composer.defaultTitle.${output}`);
  },
  { immediate: true },
);
watch(
  () => form.output,
  (output) => {
    form.target = undefined;
    if (!form.title.trim()) form.title = t(`spaces.creation.composer.defaultTitle.${output}`);
  },
);

function selectDuration(value: string) {
  const seconds = videoDurations.find((duration) => String(duration) === value);
  if (seconds) form.durationSeconds = seconds;
}
</script>

<template>
  <UiDialog
    v-model:open="open"
    :title="t('spaces.creation.composer.title')"
    :description="t('spaces.creation.composer.description')"
  >
    <div class="space-y-4">
      <label class="block space-y-1.5">
        <span class="text-label font-medium">{{ t("spaces.creation.composer.outputLabel") }}</span>
        <UiSelect
          v-model="form.output"
          :label="t('spaces.creation.composer.outputLabel')"
          :options="outputs"
        />
      </label>
      <label class="block space-y-1.5">
        <span class="text-label font-medium">{{ t("common.title") }}</span>
        <UiInput v-model="form.title" />
      </label>
      <label class="block space-y-1.5">
        <span class="text-label font-medium">{{ t("spaces.creation.composer.requirement") }}</span>
        <UiTextarea
          v-model="form.requirement"
          :rows="5"
          :placeholder="t(`spaces.creation.composer.placeholder.${form.output}`)"
        />
      </label>
      <div v-if="form.output === 'image'">
        <label class="block space-y-1.5 text-label font-medium">
          {{ t("spaces.creation.composer.aspectRatio") }}
          <UiSelect
            v-model="form.imageAspectRatio"
            :label="t('spaces.creation.composer.aspectRatio')"
            :options="imageRatios"
          />
        </label>
      </div>
      <div v-else-if="form.output === 'video'" class="grid grid-cols-2 gap-3">
        <label class="space-y-1.5 text-label font-medium">
          {{ t("spaces.creation.composer.aspectRatio") }}
          <UiSelect
            v-model="form.videoAspectRatio"
            :label="t('spaces.creation.composer.aspectRatio')"
            :options="videoRatios"
          />
        </label>
        <label class="space-y-1.5 text-label font-medium">
          {{ t("spaces.creation.composer.duration") }}
          <UiSelect
            :model-value="String(form.durationSeconds)"
            :label="t('spaces.creation.composer.duration')"
            :options="durations"
            @update:model-value="selectDuration"
          />
        </label>
      </div>
      <AiExecutionTargetField v-model="form.target" :purpose="creationTargetPurpose(form.output)" />
      <div>
        <p class="text-caption-sm text-fg-tertiary">
          {{ t("spaces.creation.composer.sources", { count: sources.length }) }}
        </p>
        <p class="mt-1 line-clamp-2 text-caption text-fg-secondary">
          {{ sources.map((source) => source.title).join(" · ") }}
        </p>
      </div>
    </div>
    <template #footer>
      <UiButton variant="ghost" @click="open = false">{{ t("common.cancel") }}</UiButton>
      <UiButton variant="primary" :disabled="!valid" @click="emit('submit', { ...form })">
        {{ t("spaces.creation.composer.create") }}
      </UiButton>
    </template>
  </UiDialog>
</template>
