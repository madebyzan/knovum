<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import type { CapturedWorkflowRun } from "../../contracts/generated/CapturedWorkflowRun";
import { useSpaceStore } from "../../session/space-store";
import { useCapturedWorkflowLibrary } from "../use-captured-workflow-library";
import { useCapturedWorkflowRun } from "../use-captured-workflow-run";
import { useWorkflowBindings } from "../use-workflow-bindings";
import { useWorkflowChoices } from "../use-workflow-choices";
import SpaceWorkflowBindings from "./SpaceWorkflowBindings.vue";
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
  retry: retryLibrary,
  renamed,
  deleted,
  addRun,
} = useCapturedWorkflowLibrary(() => store.activeSpaceId);
const workflowId = ref("");
const run = useCapturedWorkflowRun();
const requestId = ref(crypto.randomUUID());
let autoSelected = false;

const workflow = computed(() => items.value.find((item) => item.workflowId === workflowId.value));
const { slots, ready, bindings, reset: resetBindings, set } = useWorkflowBindings(workflow);
const choices = useWorkflowChoices(items);

watch(
  [
    workflowId,
    () => workflow.value?.version,
    () => store.workflowCaptureSelection.join("\u0000"),
    () => store.activeSpaceId,
  ],
  () => {
    resetBindings();
    run.reset();
    requestId.value = crypto.randomUUID();
  },
);

watch([loading, loadError], autoSelect, { immediate: true });

function autoSelect() {
  if (autoSelected || loading.value || loadError.value) return;
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
  const spaceId = store.activeSpaceId;
  if (!ready.value || !workflow.value || !spaceId) return;
  const selectedWorkflow = workflow.value;
  const selectedRequestId = requestId.value;
  const started = await run.start({
    workflow: selectedWorkflow,
    spaceId,
    bindings: bindings.value,
    requestId: selectedRequestId,
    isCurrent: () => store.activeSpaceId === spaceId && requestId.value === selectedRequestId,
  });
  if (started) addRun(spaceId, { run: started, workflowId: selectedWorkflow.workflowId });
}

async function retry() {
  const entry = history.value.find(({ run: item }) => item.id === run.runId.value);
  const spaceId = store.activeSpaceId;
  if (!entry || !spaceId) return;
  const title =
    items.value.find((item) => item.workflowId === entry.workflowId)?.name ??
    t("spaces.creation.workflow.title");
  const started = await run.retry(title);
  if (started) addRun(spaceId, { run: started, workflowId: entry.workflowId });
}

function setBinding(slotId: string, nodeIds: string[]) {
  set(slotId, nodeIds);
  requestId.value = crypto.randomUUID();
  run.error.value = "";
}

function deletedWorkflow(deletedWorkflowId: string) {
  deleted(deletedWorkflowId);
  if (workflowId.value === deletedWorkflowId) workflowId.value = "";
}
</script>

<template>
  <div class="flex h-full min-h-0 flex-col text-label">
    <div class="min-h-0 flex-1 space-y-4 overflow-y-auto px-3 py-3">
      <p v-if="loading" role="status" class="text-fg-secondary">
        {{ t("spaces.creation.workflow.derivation.loading") }}
      </p>
      <div v-else-if="loadError" class="space-y-2">
        <p role="alert" class="text-caption text-fg-danger">{{ loadError }}</p>
        <UiButton variant="secondary" @click="retryLibrary">
          {{ t("common.retry") }}
        </UiButton>
      </div>
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
          <SpaceWorkflowBindings :slots="slots" :bindings="bindings" @change="setBinding" />
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
          :active-run-id="run.runId.value"
          :active-run-state="run.runState.value"
          @select="showHistory"
        />
        <p v-if="run.error.value" role="alert" class="text-caption text-fg-danger">
          {{ run.error.value }}
        </p>
      </template>
    </div>
    <div v-if="workflow && !run.runId.value" class="border-t border-stroke-default px-3 py-3">
      <UiButton variant="primary" class="w-full" :disabled="!ready || run.submitting.value" @click="start">
        {{ t("spaces.creation.workflow.run.start") }}
      </UiButton>
    </div>
    <SpaceWorkflowRunActions
      v-else
      :run-id="run.runId.value"
      :run-state="run.runState.value"
      :cancelling="run.cancelling.value"
      :retrying="run.retrying.value"
      :allow-new="Boolean(workflow)"
      :outputs="run.outputNodes.value"
      @locate="run.locate"
      @cancel="run.cancel"
      @retry="retry"
      @new-run="newRun"
    />
  </div>
</template>
