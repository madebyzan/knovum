<script setup lang="ts">
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import type { CaptureCandidate } from "../../contracts/generated/CaptureCandidate";
import type { CapturePreview } from "../../contracts/generated/CapturePreview";
import { workflowOutputNamesValid } from "../../workflows/output-name-policy";
import { useWorkflowCaptureSave } from "../../workflows/use-workflow-capture-save";

const props = defineProps<{
  spaceId: string;
  selectedNodeIds: string[];
  preview: CapturePreview;
  candidate: CaptureCandidate;
}>();
const { t } = useI18n();
const save = useWorkflowCaptureSave();
const outputNames = ref<Record<string, string>>(
  Object.fromEntries(
    props.candidate.outputSlots.map((slot, index) => [
      slot.id,
      t("spaces.creation.workflow.outputs.name", { index: index + 1 }),
    ]),
  ),
);
const outputsNamed = computed(() => {
  const names = props.candidate.outputSlots.map((slot) => outputNames.value[slot.id]?.trim() ?? "");
  return workflowOutputNamesValid(names);
});

function setOutputName(id: string, value: string) {
  outputNames.value = { ...outputNames.value, [id]: value };
}

function submit() {
  void save.submit({
    spaceId: props.spaceId,
    selectedNodeIds: props.selectedNodeIds,
    preview: props.preview,
    outputNames: outputNames.value,
  });
}
</script>

<template>
  <form
    v-if="!save.saved.value"
    class="max-h-80 space-y-2 overflow-y-auto border-t border-stroke-default px-3 py-3"
    @submit.prevent="submit"
  >
    <label class="block text-caption text-fg-secondary" for="capture-workflow-name">{{
      t("spaces.creation.workflow.name")
    }}</label>
    <UiInput
      id="capture-workflow-name"
      v-model="save.name.value"
      :placeholder="t('spaces.creation.workflow.namePlaceholder')"
    />
    <div v-for="(output, index) in candidate.outputSlots" :key="output.id" class="space-y-1">
      <label class="block text-caption text-fg-secondary" :for="`capture-output-${output.id}`">
        {{ t("spaces.creation.workflow.outputs.name", { index: index + 1 }) }}
      </label>
      <UiInput
        :id="`capture-output-${output.id}`"
        :model-value="outputNames[output.id] ?? ''"
        @update:model-value="setOutputName(output.id, $event)"
      />
    </div>
    <p v-if="!outputsNamed" class="text-caption text-fg-warning">
      {{ t("spaces.creation.workflow.outputs.namesUnique") }}
    </p>
    <p v-if="save.error.value" class="text-caption text-fg-danger" role="alert">
      {{ save.error.value }}
    </p>
    <UiButton
      type="submit"
      variant="primary"
      class="w-full"
      :disabled="!save.name.value.trim() || !outputsNamed || save.saving.value"
    >
      {{ t("spaces.creation.workflow.saveCaptured") }}
    </UiButton>
  </form>
  <p v-else class="border-t border-stroke-default px-3 py-3 text-caption text-fg-success" role="status">
    {{ t("spaces.creation.workflow.saved") }}
  </p>
</template>
