<script setup lang="ts">
import { useI18n } from "vue-i18n";

import UiBadge from "../../../../shared/ui/UiBadge.vue";
import UiButton from "../../../../shared/ui/UiButton.vue";
import { useRunCenter } from "../../runs/use-run-center";
import SpaceExternalAgentPermissions from "./SpaceExternalAgentPermissions.vue";
import SpaceRunStepList from "./SpaceRunStepList.vue";

const { t } = useI18n();
const {
  approvals,
  batchItems,
  continueStep,
  control,
  decideApproval,
  events,
  exportDiagnostics,
  reconcile,
  refreshDetails,
  retry,
  review,
  runs,
  select,
  selectedRun,
  selectedStepIds,
  steps,
  submitInput,
  toggleStep,
} = useRunCenter();

function formatted(value: unknown) {
  return JSON.stringify(value, null, 2);
}
</script>

<template>
  <section class="grid h-full min-h-0 grid-rows-[minmax(0,1fr)_minmax(0,1fr)] bg-panel">
    <div class="overflow-y-auto border-b border-divider p-2">
      <div
        v-for="approval in approvals"
        :key="approval.id"
        class="mb-2 rounded-control border border-warning/40 bg-raised p-2"
      >
        <p class="text-label">{{ approval.actionId }}</p>
        <p class="truncate text-micro text-foreground-tertiary">{{ approval.planDigest }}</p>
        <details class="mt-1 text-micro">
          <summary class="cursor-pointer">{{ t("spaces.assistant.approvalPlan") }}</summary>
          <pre class="mt-1 max-h-48 overflow-auto rounded-control bg-control p-2 whitespace-pre-wrap">{{
            formatted(approval.plan)
          }}</pre>
        </details>
        <div class="mt-1 flex gap-1">
          <UiButton size="sm" variant="primary" @click="decideApproval(approval, true)">{{
            t("common.approve")
          }}</UiButton>
          <UiButton size="sm" variant="dangerSubtle" @click="decideApproval(approval, false)">{{
            t("common.reject")
          }}</UiButton>
        </div>
      </div>
      <div
        v-for="run in runs"
        :key="run.id"
        class="mb-1 rounded-control border border-border/70 bg-raised p-2"
        @click="select(run.id)"
      >
        <div class="flex items-center gap-2">
          <span class="min-w-0 flex-1 truncate text-label">{{ run.kind }} · {{ run.id.slice(0, 8) }}</span>
          <UiBadge size="sm">{{ run.state }}</UiBadge>
        </div>
        <div class="mt-1 flex gap-1">
          <UiButton
            v-if="run.state === 'running' && run.kind !== 'external_agent'"
            size="sm"
            variant="ghost"
            @click.stop="control('pause', run.id)"
            >{{ t("spaces.assistant.pause") }}</UiButton
          >
          <UiButton
            v-if="run.state === 'paused'"
            size="sm"
            variant="ghost"
            @click.stop="control('resume', run.id)"
            >{{ t("spaces.assistant.resume") }}</UiButton
          >
          <UiButton
            v-if="!['succeeded', 'failed', 'cancelled'].includes(run.state)"
            size="sm"
            variant="dangerSubtle"
            @click.stop="control('cancel', run.id)"
            >{{ t("common.cancel") }}</UiButton
          >
          <UiButton
            v-if="
              !['assistant', 'external_agent'].includes(run.kind) &&
              ['failed', 'completed_with_errors', 'cancelled'].includes(run.state)
            "
            size="sm"
            variant="ghost"
            @click.stop="retry(run.id)"
            >{{ t("common.retry") }}</UiButton
          >
          <UiButton size="sm" variant="ghost" @click.stop="exportDiagnostics(run.id)">
            {{ t("spaces.assistant.exportDiagnostics") }}
          </UiButton>
        </div>
        <p v-if="run.totalTokens" class="mt-1 text-micro text-foreground-tertiary">
          {{ run.totalTokens.toLocaleString() }} tokens · {{ run.inputTokens.toLocaleString() }} in ·
          {{ run.outputTokens.toLocaleString() }} out
        </p>
        <p class="mt-1 text-micro text-foreground-tertiary">
          {{ run.stepAttempts.toLocaleString() }} attempts · {{ run.expandedSteps.toLocaleString() }} expanded
          · {{ run.outputBytes.toLocaleString() }} output bytes
        </p>
      </div>
      <p v-if="runs.length === 0" class="text-meta p-6 text-center text-foreground-tertiary">
        {{ t("spaces.assistant.runEmpty") }}
      </p>
    </div>
    <div class="overflow-y-auto p-2">
      <div v-if="selectedRun" class="mb-2 rounded-control border border-border/70 bg-raised p-2 text-micro">
        <p v-if="selectedRun.parentRunId">Parent · {{ selectedRun.parentRunId }}</p>
        <p v-if="selectedRun.retryOfRunId">Retry of · {{ selectedRun.retryOfRunId }}</p>
        <p v-if="selectedRun.forkOfRunId">Fork of · {{ selectedRun.forkOfRunId }}</p>
      </div>
      <SpaceExternalAgentPermissions :events="events" @resolved="refreshDetails" />
      <SpaceRunStepList
        :steps="steps"
        :batch-items="batchItems"
        :selected-step-ids="selectedStepIds"
        :retry-selectable="
          Boolean(selectedRun && ['failed', 'completed_with_errors', 'cancelled'].includes(selectedRun.state))
        "
        :on-toggle="toggleStep"
        :on-submit-input="submitInput"
        :on-review="review"
        :on-reconcile="reconcile"
        :on-continue="continueStep"
      />
      <details v-if="events.length" class="mt-2 rounded-control border border-border/70 p-2">
        <summary class="cursor-pointer text-label">Run events · {{ events.length }}</summary>
        <ol class="mt-2 space-y-1">
          <li v-for="event in events" :key="event.sequence" class="rounded-control bg-control p-2 text-micro">
            <p>{{ event.sequence }} · {{ event.kind }}</p>
            <pre class="overflow-auto whitespace-pre-wrap text-foreground-secondary">{{
              formatted(event.payload)
            }}</pre>
          </li>
        </ol>
      </details>
    </div>
  </section>
</template>
