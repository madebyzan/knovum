<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import { useSpaceStore } from "../../session/space-store";
import type { CapturedWorkflowRun } from "../../workflows/captured-workflow-ipc";
import { useCapturedWorkflowLibrary } from "../../workflows/use-captured-workflow-library";
import { useCapturedWorkflowRun } from "../../workflows/use-captured-workflow-run";
import { useWorkflowBindings } from "../../workflows/use-workflow-bindings";
import { useWorkflowChoices } from "../../workflows/use-workflow-choices";
import SpaceWorkflowBinding from "./SpaceWorkflowBinding.vue";
import SpaceWorkflowManagement from "./SpaceWorkflowManagement.vue";
import SpaceWorkflowOutputs from "./SpaceWorkflowOutputs.vue";
import SpaceWorkflowRunActions from "./SpaceWorkflowRunActions.vue";
import SpaceWorkflowRunHistory from "./SpaceWorkflowRunHistory.vue";

const store = useSpaceStore();
const { t } = useI18n();
const {
  items,
  history,
  loading,
  error: loadError,
  renamed,
  deleted,
  addRun,
} = useCapturedWorkflowLibrary(() => store.activeSpaceId);
const workflowId = ref("");
const run = useCapturedWorkflowRun();
const { error, runId, runState, cancelling, submitting, retrying, outputNodes } = run;
const requestId = ref(crypto.randomUUID());
let autoSelected = false;

const workflow = computed(() => items.value.find((item) => item.workflowId === workflowId.value));
const binding = useWorkflowBindings(workflow);
const { slots, ready, bindings } = binding;
const choices = useWorkflowChoices(items);

watch(
  [workflowId, () => workflow.value?.version, () => store.workflowCaptureSelection.join("\u0000")],
  () => {
    binding.reset();
    run.reset();
    requestId.value = crypto.randomUUID();
  },
);

watch(loading, autoSelect, { immediate: true });

function autoSelect() {
  if (autoSelected || loading.value) return;
  autoSelected = true;
  if (choices.value.length === 1 && !choices.value[0]!.disabled)
    workflowId.value = items.value[0]!.workflowId;
}

function showHistory(entry: CapturedWorkflowRun) {
  run.reset();
  const title =
    items.value.find((item) => item.workflowId === entry.workflowId)?.name ??
    t("spaces.creation.workflow.title");
  run.attach(entry.run, title, true);
}

function newRun() {
  run.reset();
  requestId.value = crypto.randomUUID();
}

async function start() {
  if (!ready.value || !workflow.value || !store.activeSpaceId) return;
  const selectedWorkflow = workflow.value;
  const selectedRequestId = requestId.value;
  await run.start({
    workflow: selectedWorkflow,
    spaceId: store.activeSpaceId,
    bindings: bindings.value,
    requestId: selectedRequestId,
    isCurrent: () => requestId.value === selectedRequestId,
    onStarted: (started) => addRun({ run: started, workflowId: selectedWorkflow.workflowId }),
  });
}

async function retry() {
  const entry = history.value.find(({ run: item }) => item.id === runId.value);
  if (!entry) return;
  const title =
    items.value.find((item) => item.workflowId === entry.workflowId)?.name ??
    t("spaces.creation.workflow.title");
  await run.retry(title, (started) => addRun({ run: started, workflowId: entry.workflowId }));
}

function setBinding(slotId: string, nodeIds: string[]) {
  binding.set(slotId, nodeIds);
  requestId.value = crypto.randomUUID();
  error.value = "";
}

function deletedWorkflow(id: string) {
  deleted(id);
  if (workflowId.value === id) workflowId.value = "";
}
</script>

<template>
  <div class="flex h-full min-h-0 flex-col text-label">
    <div class="min-h-0 flex-1 space-y-4 overflow-y-auto px-3 py-3">
      <p v-if="loading" role="status" class="text-fg-secondary">
        {{ t("spaces.creation.workflow.derivation.loading") }}
      </p>
      <template v-else>
        <p v-if="!items.length" class="text-caption text-fg-secondary">
          {{ t("spaces.creation.workflow.empty") }}
        </p>
        <UiButton
          v-if="!items.length"
          variant="secondary"
          class="w-full"
          @click="store.openWorkflowCapture(store.workflowCaptureSelection)"
        >
          {{ t("spaces.creation.workflow.createFromSelection") }}
        </UiButton>
        <UiSelect
          v-else
          v-model="workflowId"
          :label="t('spaces.creation.workflow.select')"
          :options="choices"
          :placeholder="t('spaces.creation.workflow.select')"
        />
        <p
          v-if="items.length && choices.every((choice) => choice.disabled)"
          class="text-caption text-fg-warning"
        >
          {{ t("spaces.creation.workflow.inputs.missing") }}
        </p>
        <template v-if="workflow">
          <p class="text-caption-sm text-fg-tertiary tabular-nums">v{{ workflow.version }}</p>
          <SpaceWorkflowOutputs :outputs="workflow.candidate.outputSlots" />
          <section class="space-y-3">
            <h3 class="text-caption font-semibold text-fg-secondary">
              {{ t("spaces.creation.workflow.inputs.bind") }}
            </h3>
            <SpaceWorkflowBinding
              v-for="slot in slots"
              :key="slot.id"
              :slot="slot"
              :label="slot.label"
              :options="slot.options"
              :model-value="bindings[slot.id] ?? []"
              @update:model-value="setBinding(slot.id, $event)"
            />
            <p v-if="slots.some((slot) => !slot.options.length)" class="text-caption text-fg-warning">
              {{ t("spaces.creation.workflow.inputs.missing") }}
            </p>
          </section>
          <SpaceWorkflowManagement
            :workflow="workflow"
            @renamed="renamed"
            @revised="renamed"
            @deleted="deletedWorkflow"
          />
        </template>
        <SpaceWorkflowRunHistory
          :entries="history"
          :workflows="items"
          :active-run-id="runId"
          :active-run-state="runState"
          @select="showHistory"
        />
        <p v-if="error || loadError" role="alert" class="text-caption text-fg-danger">
          {{ error || loadError }}
        </p>
      </template>
    </div>
    <div v-if="workflow && !runId" class="border-t border-stroke-default px-3 py-3">
      <UiButton variant="primary" class="w-full" :disabled="!ready || submitting" @click="start">
        {{ t("spaces.creation.workflow.run.start") }}
      </UiButton>
    </div>
    <SpaceWorkflowRunActions
      v-else
      :run-id="runId"
      :run-state="runState"
      :cancelling="cancelling"
      :retrying="retrying"
      :allow-new="Boolean(workflow)"
      :outputs="outputNodes"
      @locate="run.locate"
      @cancel="run.cancel"
      @retry="retry"
      @new-run="newRun"
    />
  </div>
</template>
