<script setup lang="ts">
import { isString } from "es-toolkit";
import { computed, reactive, watch } from "vue";
import { useI18n } from "vue-i18n";

import type { GenerationPreset } from "../../../../foundation/ai/ai-models";
import AiExecutionTargetField from "../../../../foundation/ai/AiExecutionTargetField.vue";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiDialog from "../../../../shared/ui/UiDialog.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import UiTextarea from "../../../../shared/ui/UiTextarea.vue";
import type { GroundedDocumentSource } from "../../knowledge-derivations/catalog/grounded-document";
import { isImageAspectRatio, type VisualizationDraft } from "../../knowledge-derivations/visualization";
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
  { value: "explanatory editorial illustration", label: t("spaces.creation.styleEditorial") },
  { value: "clean information diagram", label: t("spaces.creation.styleDiagram") },
  { value: "cinematic concept art", label: t("spaces.creation.styleConcept") },
]);
const ratios = ["1:1", "3:2", "2:3", "16:9", "9:16"].map((value) => ({ value, label: value }));
watch(open, (isOpen) => {
  if (!isOpen) return;
  form.title = t("spaces.creation.visualizationDefaultTitle");
  form.intent = "";
});
function applyPreset(preset: GenerationPreset) {
  if (preset.actionId !== "generate-visualization") return;
  form.target = preset.target;
  if (isString(preset.parameters.intent)) form.intent = preset.parameters.intent;
  if (isString(preset.parameters.style)) form.style = preset.parameters.style;
  if (isImageAspectRatio(preset.parameters.aspectRatio)) {
    form.aspectRatio = preset.parameters.aspectRatio;
  }
}
</script>

<template>
  <UiDialog
    v-model:open="open"
    :title="t('spaces.creation.visualizationTitle')"
    :description="t('spaces.creation.visualizationDescription')"
  >
    <div class="space-y-4">
      <label class="block space-y-1.5"
        ><span class="text-label font-medium">{{ t("spaces.inspector.detailTitle") }}</span
        ><UiInput v-model="form.title"
      /></label>
      <label class="block space-y-1.5"
        ><span class="text-label font-medium">{{ t("spaces.creation.visualizationIntent") }}</span
        ><UiTextarea
          v-model="form.intent"
          :rows="4"
          :placeholder="t('spaces.creation.visualizationPlaceholder')"
      /></label>
      <div class="grid grid-cols-2 gap-3">
        <label class="space-y-1.5 text-label font-medium"
          >{{ t("spaces.creation.visualStyle")
          }}<UiSelect
            v-model="form.style"
            :label="t('spaces.creation.visualStyle')"
            :options="styles" /></label
        ><label class="space-y-1.5 text-label font-medium"
          >{{ t("spaces.creation.aspectRatio")
          }}<UiSelect v-model="form.aspectRatio" :label="t('spaces.creation.aspectRatio')" :options="ratios"
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
      <p class="text-caption text-foreground-tertiary">
        {{ t("spaces.creation.documentSources", { count: sources.length }) }}
      </p>
    </div>
    <template #footer
      ><UiButton variant="ghost" @click="open = false">{{ t("common.cancel") }}</UiButton
      ><UiButton variant="primary" :disabled="!valid" @click="emit('submit', { ...form })">{{
        t("spaces.creation.visualize")
      }}</UiButton></template
    >
  </UiDialog>
</template>
