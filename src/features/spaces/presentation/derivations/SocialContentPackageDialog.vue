<script setup lang="ts">
import { computed, reactive, watch } from "vue";
import { useI18n } from "vue-i18n";

import AiExecutionTargetField from "../../../../foundation/ai/AiExecutionTargetField.vue";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiDialog from "../../../../shared/ui/UiDialog.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import type { GroundedDocumentSource } from "../../knowledge-derivations/catalog/grounded-document";
import type { SocialContentPackageDraft } from "../../knowledge-derivations/workflows/social-content-package";

const open = defineModel<boolean>("open", { required: true });
const props = defineProps<{ sources: readonly GroundedDocumentSource[] }>();
const emit = defineEmits<{ submit: [draft: SocialContentPackageDraft] }>();
const { t } = useI18n();
const form = reactive<SocialContentPackageDraft>({
  title: "",
  platform: "xiaohongshu",
  audience: "",
  goal: "",
  tone: "friendly",
  imageStyle: "clean editorial social media illustration",
  aspectRatio: "9:16",
});
const valid = computed(
  () =>
    Boolean(form.title.trim() && form.audience.trim() && form.goal.trim()) &&
    Boolean(form.textTarget && form.imageTarget),
);
const platforms = computed(() => [
  { value: "xiaohongshu", label: t("spaces.creation.platformXiaohongshu") },
  { value: "linkedin", label: "LinkedIn" },
  { value: "generic", label: t("spaces.creation.platformGeneric") },
]);
const tones = computed(() => [
  { value: "friendly", label: t("spaces.creation.toneFriendly") },
  { value: "professional", label: t("spaces.creation.toneProfessional") },
  { value: "energetic", label: t("spaces.creation.toneEnergetic") },
]);
const ratios = ["1:1", "2:3", "9:16"].map((value) => ({ value, label: value }));

watch(open, (isOpen) => {
  if (!isOpen) return;
  form.title = t("spaces.creation.socialContentDefaultTitle");
  form.audience = "";
  form.goal = "";
});
</script>

<template>
  <UiDialog
    v-model:open="open"
    :title="t('spaces.creation.socialContentPackage')"
    :description="t('spaces.creation.socialContentDescription')"
  >
    <div class="space-y-4">
      <UiInput v-model="form.title" :placeholder="t('spaces.inspector.detailTitle')" />
      <div class="grid grid-cols-2 gap-3">
        <UiSelect v-model="form.platform" :label="t('spaces.creation.socialPlatform')" :options="platforms" />
        <UiSelect v-model="form.tone" :label="t('spaces.creation.socialTone')" :options="tones" />
      </div>
      <UiInput v-model="form.audience" :placeholder="t('spaces.creation.socialAudience')" />
      <UiInput v-model="form.goal" :placeholder="t('spaces.creation.socialGoal')" />
      <div class="grid grid-cols-2 gap-3">
        <UiInput v-model="form.imageStyle" :placeholder="t('spaces.creation.visualStyle')" />
        <UiSelect v-model="form.aspectRatio" :label="t('spaces.creation.aspectRatio')" :options="ratios" />
      </div>
      <AiExecutionTargetField v-model="form.textTarget" purpose="knowledge" />
      <AiExecutionTargetField v-model="form.imageTarget" purpose="image-generation" />
      <p class="text-caption text-foreground-tertiary">
        {{ t("spaces.creation.documentSources", { count: sources.length }) }}
      </p>
    </div>
    <template #footer>
      <UiButton variant="ghost" @click="open = false">{{ t("common.cancel") }}</UiButton>
      <UiButton variant="primary" :disabled="!valid" @click="emit('submit', { ...form })">
        {{ t("spaces.creation.createContentKit") }}
      </UiButton>
    </template>
  </UiDialog>
</template>
