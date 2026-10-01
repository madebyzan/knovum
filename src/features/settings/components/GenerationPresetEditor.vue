<script setup lang="ts">
import { cloneDeep } from "es-toolkit";
import { computed, onMounted, ref, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import AiExecutionTargetField from "../../../foundation/ai/AiExecutionTargetField.vue";
import type { GenerationPreset } from "../../../foundation/ai/schema/models";
import type { RoutingPurpose } from "../../../shared/contracts/generated/RoutingPurpose";
import UiButton from "../../../shared/ui/UiButton.vue";
import UiDialog from "../../../shared/ui/UiDialog.vue";
import UiInput from "../../../shared/ui/UiInput.vue";
import type { GenerationActionOption } from "../generation-action-option";
import CreationParameterFields from "./CreationParameterFields.vue";

const open = defineModel<boolean>("open", { required: true });
const props = defineProps<{
  preset: GenerationPreset;
  loadGenerationActions: () => Promise<GenerationActionOption[]>;
}>();
const emit = defineEmits<{ save: [preset: GenerationPreset] }>();
const { t } = useI18n();
const draft = ref<GenerationPreset>(cloneDeep(props.preset));
const actions = shallowRef<GenerationActionOption[]>([]);
const actionLoadError = shallowRef<unknown>();

onMounted(async () => {
  try {
    actions.value = await props.loadGenerationActions();
  } catch (error) {
    actionLoadError.value = error;
  }
});

watch(open, (value) => {
  if (value) draft.value = cloneDeep(props.preset);
});

const action = computed(() => actions.value.find(({ id }) => id === draft.value.actionId));
const routingPurpose = computed<RoutingPurpose>(() => {
  if (!action.value?.routingPurpose) {
    throw new Error(`SpaceAction has no execution target: ${draft.value.actionId}`);
  }
  return action.value.routingPurpose;
});
const valid = computed(() => Boolean(action.value?.routingPurpose && draft.value.name.trim()));

function save() {
  if (!valid.value) return;
  emit("save", { ...cloneDeep(draft.value), name: draft.value.name.trim() });
  open.value = false;
}
</script>

<template>
  <UiDialog v-model:open="open" :title="t('settings.creation.presets.edit')">
    <div class="grid gap-4">
      <label class="grid gap-1.5 text-label font-medium">
        {{ t("settings.creation.presets.name") }}
        <UiInput v-model="draft.name" />
      </label>
      <p v-if="actionLoadError" class="text-caption text-fg-danger">
        {{ String(actionLoadError) }}
      </p>
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
