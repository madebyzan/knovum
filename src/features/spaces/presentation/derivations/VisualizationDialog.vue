<script setup lang="ts">
import { isString } from "es-toolkit";
import { computed, reactive, watch } from "vue";
import { useI18n } from "vue-i18n";

import AiExecutionTargetField from "../../../../foundation/ai/AiExecutionTargetField.vue";
import type { GenerationPreset } from "../../../../foundation/ai/schema/models";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiDialog from "../../../../shared/ui/UiDialog.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import UiTextarea from "../../../../shared/ui/UiTextarea.vue";
import type { GroundedDocumentSource } from "../../knowledge-derivations/catalog/grounded-document";
import { imageAspectRatioSchema, type VisualizationDraft } from "../../knowledge-derivations/visualization";
import GenerationPresetField from "./GenerationPresetField.vue";

const open = defineModel<boolean>("open", { required: true });
const props = defineProps<{ sources: readonly GroundedDocumentSource[] }>();
const emit = defineEmits<{ submit: [draft: VisualizationDraft] }>();
const { t } = useI18n();
const form = reactive<VisualizationDraft>({
  title: "",
  intent: "",
  style: "explanatory editorial illustration",
  aspectRatio: "16:9",
});
const presetParameters = computed(() => ({
  intent: form.intent,
  style: form.style,
  aspectRatio: form.aspectRatio,
}));
const valid = computed(() => Boolean(form.title.trim() && form.intent.trim() && form.target));
const styles = computed(() => [
  { value: "explanatory editorial illustration", label: t("spaces.creation.image.styles.editorial") },
  { value: "clean information diagram", label: t("spaces.creation.image.styles.diagram") },
  { value: "cinematic concept art", label: t("spaces.creation.image.styles.concept") },
]);
const ratios = imageAspectRatioSchema.options.map((value) => ({ value, label: value }));
watch(open, (isOpen) => {
  if (!isOpen) return;
  form.title = t("spaces.creation.image.visualization.defaultTitle");
  form.intent = "";
});
function applyPreset(preset: GenerationPreset) {
  if (preset.actionId !== "generate-visualization") return;
  form.target = preset.target;
  if (isString(preset.parameters.intent)) form.intent = preset.parameters.intent;
  if (isString(preset.parameters.style)) form.style = preset.parameters.style;
  const aspectRatio = imageAspectRatioSchema.safeParse(preset.parameters.aspectRatio);
  if (aspectRatio.success) form.aspectRatio = aspectRatio.data;
}
</script>

<template>
  <UiDialog
    v-model:open="open"
    :title="t('spaces.creation.image.visualization.title')"
    :description="t('spaces.creation.image.visualization.description')"
  >
    <div class="space-y-4">
      <label class="block space-y-1.5"
        ><span class="text-label font-medium">{{ t("common.title") }}</span
        ><UiInput v-model="form.title"
      /></label>
      <label class="block space-y-1.5"
        ><span class="text-label font-medium">{{ t("spaces.creation.image.visualization.intent") }}</span
        ><UiTextarea
          v-model="form.intent"
          :rows="4"
          :placeholder="t('spaces.creation.image.visualization.placeholder')"
      /></label>
      <div class="grid grid-cols-2 gap-3">
        <label class="space-y-1.5 text-label font-medium"
          >{{ t("spaces.creation.image.style")
          }}<UiSelect
            v-model="form.style"
            :label="t('spaces.creation.image.style')"
            :options="styles" /></label
        ><label class="space-y-1.5 text-label font-medium"
          >{{ t("spaces.creation.image.aspectRatio")
          }}<UiSelect
            v-model="form.aspectRatio"
            :label="t('spaces.creation.image.aspectRatio')"
            :options="ratios"
        /></label>
      </div>
      <AiExecutionTargetField v-model="form.target" purpose="image-generation" />
      <GenerationPresetField
        action-id="generate-visualization"
        :name="form.title"
        :target="form.target"
        :parameters="presetParameters"
        @apply="applyPreset"
      />
      <p class="text-caption text-fg-tertiary">
        {{ t("spaces.creation.document.sources", { count: sources.length }) }}
      </p>
    </div>
    <template #footer
      ><UiButton variant="ghost" @click="open = false">{{ t("common.cancel") }}</UiButton
      ><UiButton variant="primary" :disabled="!valid" @click="emit('submit', { ...form })">{{
        t("spaces.creation.image.visualization.title")
      }}</UiButton></template
    >
  </UiDialog>
</template>
