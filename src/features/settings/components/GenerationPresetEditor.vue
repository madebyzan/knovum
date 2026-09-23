<script setup lang="ts">
import { computed, onMounted, ref, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import type { AiRoutingPurpose, GenerationPreset } from "../../../foundation/ai/ai-models";
import AiExecutionTargetField from "../../../foundation/ai/AiExecutionTargetField.vue";
import { listSpaceActions, type SpaceActionDefinition } from "../../../foundation/ai/space-action-catalog";
import UiButton from "../../../shared/ui/UiButton.vue";
import UiDialog from "../../../shared/ui/UiDialog.vue";
import UiInput from "../../../shared/ui/UiInput.vue";
import CreationParameterFields from "./CreationParameterFields.vue";

const open = defineModel<boolean>("open", { required: true });
const props = defineProps<{ preset: GenerationPreset }>();
const emit = defineEmits<{ save: [preset: GenerationPreset] }>();
const { t } = useI18n();
const draft = ref<GenerationPreset>(structuredClone(props.preset));
const actions = shallowRef<SpaceActionDefinition[]>([]);
const actionLoadError = shallowRef<unknown>();

onMounted(async () => {
  try {
    actions.value = await listSpaceActions();
  } catch (error) {
    actionLoadError.value = error;
  }
});

watch(open, (value) => {
  if (value) draft.value = structuredClone(props.preset);
});

const action = computed(() => actions.value.find(({ id }) => id === draft.value.actionId));
const routingPurpose = computed<AiRoutingPurpose>(() => {
  if (!action.value?.routingPurpose) {
    throw new Error(`SpaceAction has no execution target: ${draft.value.actionId}`);
  }
  return action.value.routingPurpose;
});
const valid = computed(() => Boolean(action.value?.routingPurpose && draft.value.name.trim()));

function save() {
  if (!valid.value) return;
  emit("save", { ...structuredClone(draft.value), name: draft.value.name.trim() } as GenerationPreset);
  open.value = false;
}
</script>

<template>
  <UiDialog v-model:open="open" :title="t('settings.creation.editPreset')">
    <div class="grid gap-4">
      <label class="grid gap-1.5 text-label font-medium">
        {{ t("settings.creation.presetName") }}
        <UiInput v-model="draft.name" />
      </label>
      <p v-if="actionLoadError" class="text-caption text-danger">{{ String(actionLoadError) }}</p>
      <AiExecutionTargetField v-else-if="action" v-model="draft.target" :purpose="routingPurpose" />
      <section v-if="Object.keys(draft.parameters).length" class="grid gap-2">
        <h3 class="text-label font-medium">{{ t("settings.creation.parameters") }}</h3>
        <CreationParameterFields v-model="draft.parameters" />
      </section>
    </div>
    <template #footer>
      <UiButton variant="ghost" @click="open = false">{{ t("common.cancel") }}</UiButton>
      <UiButton variant="primary" :disabled="!valid" @click="save">{{ t("common.save") }}</UiButton>
    </template>
  </UiDialog>
</template>
