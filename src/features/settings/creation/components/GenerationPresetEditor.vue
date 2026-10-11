<script setup lang="ts">
import { cloneDeep, isEqual } from "es-toolkit";
import { computed, onMounted, ref, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import AiExecutionTargetField from "../../../../foundation/ai/AiExecutionTargetField.vue";
import type { GenerationPreset } from "../../../../foundation/ai/schema/models";
import { getErrorMessage } from "../../../../foundation/errors/error-format";
import type { RoutingPurpose } from "../../../../shared/contracts/generated/RoutingPurpose";
import { confirmAction } from "../../../../shared/ui/confirm-action";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiDialog from "../../../../shared/ui/UiDialog.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import { useSettingsLeaveDecision } from "../../navigation/use-settings-leave-decision";
import type { GenerationActionOption } from "../action-option";
import CreationParameterFields from "./CreationParameterFields.vue";

const open = defineModel<boolean>("open", { required: true });
const props = defineProps<{
  preset: GenerationPreset;
  loadGenerationActions: () => Promise<GenerationActionOption[]>;
  savePreset: (preset: GenerationPreset) => Promise<void>;
}>();
const { t } = useI18n();
const draft = ref<GenerationPreset>(cloneDeep(props.preset));
const actions = shallowRef<GenerationActionOption[]>([]);
const actionLoadError = shallowRef<unknown>();
const saveError = shallowRef<unknown>();
const saving = shallowRef(false);
let admittedSave: Promise<boolean> | undefined;
const changed = computed(() => !isEqual(draft.value, props.preset));
const dialogOpen = computed({
  get: () => open.value,
  set: (value: boolean) => {
    if (!value) void requestClose();
  },
});

onMounted(async () => {
  try {
    actions.value = await props.loadGenerationActions();
  } catch (error) {
    actionLoadError.value = error;
  }
});

watch(open, (value) => {
  if (value) {
    draft.value = cloneDeep(props.preset);
    saveError.value = undefined;
  }
});

const action = computed(() => actions.value.find(({ id }) => id === draft.value.actionId));
const routingPurpose = computed<RoutingPurpose>(() => {
  if (!action.value?.routingPurpose) {
    throw new Error(`SpaceAction has no execution target: ${draft.value.actionId}`);
  }
  return action.value.routingPurpose;
});
const valid = computed(() => Boolean(action.value?.routingPurpose && draft.value.name.trim()));

async function requestClose() {
  if (admittedSave) {
    const succeeded = await admittedSave;
    if (!succeeded) return false;
    if (!open.value) return true;
  }
  if (
    changed.value &&
    !(await confirmAction({
      title: t("settings.creation.presets.discard.title"),
      description: t("settings.creation.presets.discard.description"),
      confirmLabel: t("common.discardChanges"),
      danger: true,
    }))
  )
    return false;
  open.value = false;
  return true;
}
useSettingsLeaveDecision({
  async prepare() {
    if (admittedSave && !(await admittedSave)) return false;
    if (!open.value || !changed.value) return true;
    return confirmAction({
      title: t("settings.creation.presets.discard.title"),
      description: t("settings.creation.presets.discard.description"),
      confirmLabel: t("common.discardChanges"),
      danger: true,
    });
  },
  finalize() {
    open.value = false;
  },
});

async function save() {
  if (!valid.value || saving.value) return;
  saving.value = true;
  saveError.value = undefined;
  const preset = { ...cloneDeep(draft.value), name: draft.value.name.trim() };
  const admitted: Promise<boolean> = props
    .savePreset(preset)
    .then(() => {
      open.value = false;
      return true;
    })
    .catch((error) => {
      saveError.value = error;
      return false;
    })
    .finally(() => {
      saving.value = false;
      if (admittedSave === admitted) admittedSave = undefined;
    });
  admittedSave = admitted;
  await admitted;
}
</script>

<template>
  <UiDialog v-model:open="dialogOpen" :title="t('settings.creation.presets.edit')" :closable="!saving">
    <fieldset :disabled="saving" class="grid gap-4 border-0 p-0">
      <label class="grid gap-1.5 text-label font-medium">
        {{ t("settings.creation.presets.name") }}
        <UiInput v-model="draft.name" />
      </label>
      <p v-if="actionLoadError" class="text-caption text-fg-danger">
        {{ getErrorMessage(actionLoadError) }}
      </p>
      <AiExecutionTargetField v-else-if="action" v-model="draft.target" :purpose="routingPurpose" />
      <section v-if="Object.keys(draft.parameters).length" class="grid gap-2">
        <h3 class="text-label font-medium">{{ t("settings.creation.parameters") }}</h3>
        <CreationParameterFields v-model="draft.parameters" />
      </section>
    </fieldset>
    <p v-if="saveError" class="mt-3 text-caption text-fg-danger" role="alert">
      {{ t("settings.creation.saveFailed") }}: {{ getErrorMessage(saveError) }}
    </p>
    <template #footer>
      <UiButton variant="ghost" :disabled="saving" @click="requestClose">{{ t("common.cancel") }}</UiButton>
      <UiButton variant="primary" :disabled="!valid || saving" @click="save">{{
        saving ? t("common.saving") : t("common.save")
      }}</UiButton>
    </template>
  </UiDialog>
</template>
