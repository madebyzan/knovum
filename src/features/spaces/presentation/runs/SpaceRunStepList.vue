<script setup lang="ts">
import { reactive } from "vue";
import { useI18n } from "vue-i18n";

import UiBadge from "../../../../shared/ui/UiBadge.vue";
import UiButton from "../../../../shared/ui/UiButton.vue";
import type { RunBatchItem, RunStep } from "../../runs/run-client";

const props = defineProps<{
  steps: RunStep[];
  batchItems: RunBatchItem[];
  retrySelectable: boolean;
  selectedStepIds: string[];
  onToggle: (stepId: string) => void;
  onSubmitInput: (step: RunStep, value: unknown) => Promise<void>;
  onReview: (step: RunStep, approved: boolean) => Promise<void>;
  onReconcile: (step: RunStep, decision: "retry" | "fail") => Promise<void>;
  onContinue: (step: RunStep) => Promise<void>;
}>();
const { t } = useI18n();
const inputValues = reactive<Record<string, string>>({});

function formatted(value: unknown) {
  return JSON.stringify(value, null, 2);
}

async function submit(step: RunStep) {
  const source = inputValues[step.id] ?? "";
  let value: unknown;
  try {
    value = JSON.parse(source || "null");
  } catch {
    value = source;
  }
  await props.onSubmitInput(step, value);
}

function itemsFor(step: RunStep) {
  return props.batchItems.filter(({ definitionNodeId }) => definitionNodeId === step.definitionNodeId);
}
</script>

<template>
  <ol>
    <li v-for="step in steps" :key="step.id" class="mb-1 rounded-control bg-control px-2 py-1.5">
      <div class="flex items-center gap-2">
        <input
          v-if="retrySelectable"
          type="checkbox"
          :checked="selectedStepIds.includes(step.id)"
          :aria-label="`Retry ${step.definitionNodeId}`"
          @change="onToggle(step.id)"
        />
        <span class="min-w-0 flex-1 truncate text-label">{{ step.definitionNodeId }}</span>
        <span class="text-micro text-foreground-tertiary">{{ step.nodeKind }}</span>
        <UiBadge size="sm">{{ step.state }}</UiBadge>
      </div>
      <details class="mt-1 text-micro">
        <summary class="cursor-pointer text-foreground-secondary">Input / output</summary>
        <pre class="mt-1 overflow-auto rounded-control bg-raised p-2 whitespace-pre-wrap">{{
          formatted(step.resolvedInput)
        }}</pre>
        <pre
          v-if="step.output !== null"
          class="mt-1 overflow-auto rounded-control bg-raised p-2 whitespace-pre-wrap"
          >{{ formatted(step.output) }}</pre>
      </details>
      <details v-if="itemsFor(step).length" class="mt-1 text-micro">
        <summary class="cursor-pointer text-foreground-secondary">
          Batch items · {{ itemsFor(step).length }}
        </summary>
        <ol class="mt-1 space-y-1">
          <li
            v-for="item in itemsFor(step)"
            :key="item.itemKey"
            class="flex items-center gap-2 rounded-control bg-raised px-2 py-1"
          >
            <span class="min-w-0 flex-1 truncate">{{ item.itemKey }}</span>
            <span class="text-foreground-tertiary">{{ item.attemptCount }} attempts</span>
            <UiBadge size="sm">{{ item.state }}</UiBadge>
          </li>
        </ol>
      </details>
      <div v-if="step.waitReason === 'user_input'" class="mt-2 flex gap-1">
        <input
          v-model="inputValues[step.id]"
          class="min-w-0 flex-1 rounded-control border border-border bg-raised px-2 text-label"
          placeholder="Value or JSON"
        />
        <UiButton size="sm" variant="primary" @click="submit(step)">Submit</UiButton>
      </div>
      <div v-else-if="step.waitReason === 'debug_breakpoint'" class="mt-2">
        <UiButton size="sm" variant="primary" @click="onContinue(step)">{{
          t("spaces.assistant.continueStep")
        }}</UiButton>
      </div>
      <div v-else-if="step.waitReason === 'content_review'" class="mt-2 flex gap-1">
        <UiButton size="sm" variant="primary" @click="onReview(step, true)">Approve</UiButton>
        <UiButton size="sm" variant="dangerSubtle" @click="onReview(step, false)">Reject</UiButton>
      </div>
      <div v-else-if="step.waitReason === 'external_reconciliation'" class="mt-2 flex gap-1">
        <UiButton size="sm" variant="primary" @click="onReconcile(step, 'retry')">{{
          t("common.retry")
        }}</UiButton>
        <UiButton size="sm" variant="dangerSubtle" @click="onReconcile(step, 'fail')">{{
          t("spaces.assistant.markFailed")
        }}</UiButton>
      </div>
    </li>
  </ol>
</template>
