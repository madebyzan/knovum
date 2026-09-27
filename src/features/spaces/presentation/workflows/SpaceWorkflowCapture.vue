<script setup lang="ts">
import { useQuery } from "@tanstack/vue-query";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import { useSpaceStore } from "../../session/space-store";
import {
  previewWorkflowCapture,
  type WorkflowCaptureIssueReason,
} from "../../workflows/captured-workflow-ipc";
import SpaceWorkflowSaveForm from "./SpaceWorkflowSaveForm.vue";
import { captureActionLabel, captureIssueLabels } from "./workflow-capture-labels";

const store = useSpaceStore();
const { t } = useI18n();
// The preview is derived from live canvas state, so it must never be served from cache.
const capturePreviewQuery = useQuery(
  computed(() => ({
    queryKey: [
      "spaces",
      store.activeSpaceId ?? "",
      "workflow-capture-preview",
      store.workflowCaptureSelection,
    ] as const,
    queryFn: ({ queryKey }) => previewWorkflowCapture(queryKey[1], queryKey[2]),
    enabled: Boolean(store.activeSpaceId) && store.workflowCaptureSelection.length > 0,
    staleTime: 0,
    gcTime: 0,
  })),
);
const preview = computed(() => capturePreviewQuery.data.value);
const loading = computed(() => capturePreviewQuery.isFetching.value);
const error = computed(() => {
  const cause = capturePreviewQuery.error.value;
  return cause ? String(cause) : "";
});

const nodeTitles = computed(() => new Map(store.nodes.map(({ id, data }) => [id, data.title])));
function title(nodeId: string) {
  return nodeTitles.value.get(nodeId) ?? nodeId;
}

function issueText(reason: WorkflowCaptureIssueReason) {
  return t(`spaces.creation.workflow.captureIssues.${captureIssueLabels[reason]}`);
}

function actionText(actionId: string) {
  return captureActionLabel(actionId, t);
}
</script>

<template>
  <div class="flex h-full min-h-0 flex-col text-label">
    <div class="min-h-0 flex-1 overflow-y-auto px-3 py-3">
      <p v-if="loading" class="text-foreground-secondary" role="status">
        {{ t("spaces.creation.workflow.derivation.loading") }}
      </p>
      <p v-else-if="error" class="text-danger" role="alert">{{ error }}</p>
      <template v-else-if="preview">
        <section v-if="preview.selectionGroups.length > 1" class="mb-4">
          <p class="bg-warning-subtle mb-2 rounded-control p-2 text-warning">
            {{
              t("spaces.creation.workflow.derivation.separateChains", {
                count: preview.selectionGroups.length,
              })
            }}
          </p>
          <div class="space-y-1">
            <UiButton
              v-for="(group, index) in preview.selectionGroups"
              :key="group.join(':')"
              size="sm"
              class="w-full justify-start"
              @click="store.workflowCaptureSelection = group"
            >
              {{ t("spaces.creation.workflow.derivation.chain", { index: index + 1 }) }} ·
              <span class="min-w-0 truncate">{{ group.map(title).join(" · ") }}</span>
            </UiButton>
          </div>
        </section>
        <section v-if="preview.steps.length" class="space-y-2">
          <h3 class="text-caption font-semibold text-foreground-secondary">
            {{ t("spaces.creation.workflow.derivation.steps") }}
          </h3>
          <ol class="space-y-1">
            <li v-for="(step, index) in preview.steps" :key="step.derivationId">
              <UiButton
                variant="secondary"
                class="h-auto w-full min-w-0 flex-col items-stretch border border-border py-2 text-left shadow-none"
                @click="store.requestReveal([step.outputNodeId], 'ensure-visible')"
              >
                <span class="flex items-baseline gap-2">
                  <span class="text-micro text-foreground-tertiary">{{ index + 1 }}</span>
                  <span class="min-w-0 flex-1 truncate font-medium">{{ title(step.outputNodeId) }}</span>
                </span>
                <span class="pl-4 text-caption font-normal text-foreground-secondary">{{
                  actionText(step.actionId)
                }}</span>
              </UiButton>
            </li>
          </ol>
        </section>
        <section v-if="preview.inputs.length" class="mt-4">
          <h3 class="text-caption font-semibold text-foreground-secondary">
            {{ t("spaces.creation.workflow.derivation.inputs") }}
          </h3>
          <ul class="mt-2 space-y-1">
            <li v-for="input in preview.inputs" :key="input.nodeId">
              <UiButton
                variant="ghost"
                class="w-full min-w-0 justify-start truncate bg-control"
                @click="store.requestReveal([input.nodeId], 'ensure-visible')"
              >
                <span class="truncate">{{ title(input.nodeId) }}</span>
              </UiButton>
            </li>
          </ul>
        </section>
        <section v-if="preview.issues.length" class="mt-4">
          <h3 class="text-caption font-semibold text-foreground-secondary">
            {{ t("spaces.creation.workflow.derivation.issues") }}
          </h3>
          <ul class="mt-2 space-y-1">
            <li
              v-for="issue in preview.issues"
              :key="`${issue.nodeId}:${issue.reason}`"
              class="rounded-control bg-control px-2.5 py-2"
            >
              <span class="font-medium">{{ title(issue.nodeId) }}</span>
              <p class="text-caption text-foreground-secondary">{{ issueText(issue.reason) }}</p>
            </li>
          </ul>
        </section>
        <p v-if="!preview.steps.length && !preview.issues.length" class="text-foreground-secondary">
          {{ t("spaces.creation.workflow.derivation.empty") }}
        </p>
      </template>
    </div>
    <SpaceWorkflowSaveForm
      v-if="preview?.candidate && store.activeSpaceId"
      :key="preview.sourceDigest ?? ''"
      :space-id="store.activeSpaceId"
      :selected-node-ids="store.workflowCaptureSelection"
      :preview="preview"
      :candidate="preview.candidate"
    />
  </div>
</template>
