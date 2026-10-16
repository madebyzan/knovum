<script setup lang="ts">
import { computed, reactive, watch } from "vue";
import { useI18n } from "vue-i18n";

import AiExecutionTargetField from "../../../../foundation/ai/AiExecutionTargetField.vue";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiDialog from "../../../../shared/ui/UiDialog.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import UiTextarea from "../../../../shared/ui/UiTextarea.vue";
import type { GroundedDocumentSource } from "../catalog/grounded-document";
import type { GroundedDocumentDraft } from "../grounded-document";

const open = defineModel<boolean>("open", { required: true });
const props = defineProps<{ sources: readonly GroundedDocumentSource[] }>();
const emit = defineEmits<{ submit: [draft: GroundedDocumentDraft] }>();
const { t } = useI18n();
const form = reactive<GroundedDocumentDraft>({ title: "", purpose: "" });
const valid = computed(() => Boolean(form.title.trim() && form.purpose.trim() && form.target));

watch(open, (isOpen) => {
  if (!isOpen) return;
  form.title = t("spaces.creation.document.defaultTitle");
  form.purpose = "";
});
function submit() {
  if (valid.value) emit("submit", { ...form });
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
    </div>
    <template #footer
      ><UiButton variant="ghost" @click="open = false">{{ t("common.cancel") }}</UiButton
      ><UiButton variant="primary" :disabled="!valid" @click="submit">{{
        t("spaces.creation.document.generate")
      }}</UiButton></template
    >
  </UiDialog>
</template>
