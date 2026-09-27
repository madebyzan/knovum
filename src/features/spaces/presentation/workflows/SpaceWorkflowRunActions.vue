<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import { workflowStateLabelKey } from "../../workflows/workflow-state-label";

const props = defineProps<{
  runId: string;
  runState: string;
  cancelling: boolean;
  retrying: boolean;
  allowNew: boolean;
  outputs: { outputSlotId: string; name: string; nodeId: string }[];
}>();
const emit = defineEmits<{
  locate: [nodeId: string];
  cancel: [];
  retry: [];
  newRun: [];
}>();
const { t } = useI18n();
const terminal = computed(() =>
  ["succeeded", "completed_with_errors", "failed", "cancelled"].includes(props.runState),
);
const stateText = computed(() => t(workflowStateLabelKey(props.cancelling ? "cancelling" : props.runState)));
const statusTone = computed(() => {
  if (props.runState === "failed") return "text-fg-danger";
  if (props.runState === "completed_with_errors") return "text-fg-warning";
  if (props.runState === "succeeded") return "text-fg-success";
  return "text-fg-secondary";
});
</script>

<template>
  <template v-if="runId">
    <div class="max-h-40 space-y-2 overflow-y-auto px-3 py-3">
      <p role="status" class="text-caption-md" :class="statusTone">
        {{ t("spaces.creation.workflow.run.state", { state: stateText }) }}
      </p>
      <UiButton
        v-for="output in outputs"
        :key="output.outputSlotId"
        variant="secondary"
        class="w-full justify-start"
        @click="emit('locate', output.nodeId)"
      >
        {{ t("spaces.creation.workflow.outputs.locate", { name: output.name }) }}
      </UiButton>
    </div>
    <div class="border-t border-stroke-default px-3 py-3">
      <UiButton
        v-if="!terminal"
        variant="secondary"
        class="w-full"
        :disabled="cancelling"
        @click="emit('cancel')"
      >
        {{ t("spaces.creation.workflow.run.cancel") }}
      </UiButton>
      <template v-else>
        <UiButton
          v-if="runState === 'failed' || runState === 'completed_with_errors'"
          variant="secondary"
          class="w-full"
          :disabled="retrying"
          @click="emit('retry')"
          >{{ t("spaces.creation.workflow.run.retryFailedSteps") }}</UiButton
        >
        <UiButton v-if="allowNew" variant="ghost" class="mt-2 w-full" @click="emit('newRun')">
          {{ t("spaces.creation.workflow.run.new") }}
        </UiButton>
      </template>
    </div>
  </template>
</template>
