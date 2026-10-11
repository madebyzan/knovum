<script setup lang="ts">
import { isString } from "es-toolkit";
import { computed, reactive, watch } from "vue";
import { useI18n } from "vue-i18n";

import AiExecutionTargetField from "../../../../foundation/ai/AiExecutionTargetField.vue";
import type { GenerationPreset } from "../../../../foundation/ai/schema/models";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiDialog from "../../../../shared/ui/UiDialog.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import UiTextarea from "../../../../shared/ui/UiTextarea.vue";
import type { GroundedDocumentSource } from "../../knowledge-derivations/catalog/grounded-document";
import type { GroundedDocumentDraft } from "../../knowledge-derivations/grounded-document";
import GenerationPresetField from "./GenerationPresetField.vue";

const open = defineModel<boolean>("open", { required: true });
const props = defineProps<{ sources: readonly GroundedDocumentSource[] }>();
const emit = defineEmits<{ submit: [draft: GroundedDocumentDraft] }>();
const { t } = useI18n();
const form = reactive<GroundedDocumentDraft>({ title: "", purpose: "" });
const valid = computed(() => Boolean(form.title.trim() && form.purpose.trim() && form.target));
const presetParameters = computed(() => ({ purpose: form.purpose }));

watch(open, (isOpen) => {
  if (!isOpen) return;
  form.title = t("spaces.creation.document.defaultTitle");
  form.purpose = "";
});
function submit() {
  if (valid.value) emit("submit", { ...form });
}
function applyPreset(preset: GenerationPreset) {
  if (preset.actionId !== "draft-document") return;
  form.target = preset.target;
  if (isString(preset.parameters.purpose)) form.purpose = preset.parameters.purpose;
}
</script>

<template>
  <UiDialog
    v-model:open="open"
    :title="t('spaces.creation.document.title')"
    :description="t('spaces.creation.document.description')"
  >
    <div class="space-y-4">
      <label class="block space-y-1.5"
        ><span class="text-label font-medium">{{ t("common.title") }}</span
        ><UiInput v-model="form.title"
      /></label>
      <label class="block space-y-1.5"
        ><span class="text-label font-medium">{{ t("spaces.creation.document.purpose.label") }}</span
        ><UiTextarea
          v-model="form.purpose"
          :rows="5"
          :placeholder="t('spaces.creation.document.purpose.placeholder')"
      /></label>
      <div>
        <p class="text-caption-sm text-fg-tertiary">
          {{ t("spaces.creation.document.sources", { count: sources.length }) }}
        </p>
        <p class="mt-1 line-clamp-2 text-caption text-fg-secondary">
          {{ sources.map((source) => source.title).join(" · ") }}
        </p>
      </div>
      <AiExecutionTargetField v-model="form.target" purpose="knowledge" />
      <GenerationPresetField
        action-id="draft-document"
        :name="form.title"
        :target="form.target"
        :parameters="presetParameters"
        @apply="applyPreset"
      />
    </div>
    <template #footer
      ><UiButton variant="ghost" @click="open = false">{{ t("common.cancel") }}</UiButton
      ><UiButton variant="primary" :disabled="!valid" @click="submit">{{
        t("spaces.creation.document.generate")
      }}</UiButton></template
    >
  </UiDialog>
</template>
