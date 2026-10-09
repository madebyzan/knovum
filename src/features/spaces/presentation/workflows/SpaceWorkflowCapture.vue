<script setup lang="ts">
import { useQuery } from "@tanstack/vue-query";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import UiButton from "../../../../shared/ui/UiButton.vue";
import type { CaptureIssueReason } from "../../contracts/generated/CaptureIssueReason";
import { useSpaceStore } from "../../session/space-store";
import { previewWorkflowCapture } from "../../workflows/captured-workflow-ipc";
import SpaceWorkflowSaveForm from "./SpaceWorkflowSaveForm.vue";
import { captureActionLabel, captureIssueLabels } from "./workflow-capture-labels";

const store = useSpaceStore();
const { t } = useI18n();
// The preview is derived from live canvas state, so it must never be served from cache.
const {
  data: preview,
  isFetching: loading,
  error: previewError,
} = useQuery(
  computed(() => {
    const spaceId = store.activeSpaceId ?? "";
    const selection = [...store.workflowCaptureSelection];
    return {
      queryKey: ["spaces", spaceId, "workflow-capture-preview", selection] as const,
      queryFn: () => previewWorkflowCapture(spaceId, selection),
      enabled: Boolean(spaceId) && selection.length > 0,
      staleTime: 0,
      gcTime: 0,
    };
  }),
);
const error = computed(() => {
  const cause = previewError.value;
  return cause ? getErrorMessage(cause) : "";
});

const nodeTitles = computed(() => new Map(store.nodes.map(({ id, data }) => [id, data.title])));
function title(nodeId: string) {
  return nodeTitles.value.get(nodeId) ?? nodeId;
}

function issueText(reason: CaptureIssueReason) {
  return t(`spaces.creation.workflow.captureIssues.${captureIssueLabels[reason]}`);
}

function actionText(actionId: string) {
  return captureActionLabel(actionId, t);
}
</script>

<template>
  <div class="flex h-full min-h-0 flex-col text-label">
    <div class="min-h-0 flex-1 overflow-y-auto px-3 py-3">
      <p v-if="loading" class="text-fg-secondary" role="status">
        {{ t("spaces.creation.workflow.derivation.loading") }}
      </p>
      <p v-else-if="error" class="text-fg-danger" role="alert">{{ error }}</p>
      <template v-else-if="preview">
        <section v-if="preview.selectionGroups.length > 1" class="mb-4">
          <p class="mb-2 rounded-md bg-surface-warning-subtle p-2 text-fg-warning">
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
          <h3 class="text-caption font-semibold text-fg-secondary">
            {{ t("spaces.creation.workflow.derivation.steps") }}
          </h3>
          <ol class="space-y-1">
            <li v-for="(step, index) in preview.steps" :key="step.derivationId">
              <UiButton
                variant="secondary"
                class="h-auto w-full min-w-0 flex-col items-stretch border border-stroke-default py-2 text-left shadow-none"
                @click="store.requestReveal([step.outputNodeId], 'ensure-visible')"
              >
                <span class="flex items-baseline gap-2">
                  <span class="text-caption-sm text-fg-tertiary">{{ index + 1 }}</span>
                  <span class="min-w-0 flex-1 truncate font-medium">{{ title(step.outputNodeId) }}</span>
                </span>
                <span class="pl-4 text-caption font-normal text-fg-secondary">{{
                  actionText(step.actionId)
                }}</span>
                <span class="truncate pl-4 text-caption-sm font-normal text-fg-tertiary">{{
                  t("spaces.creation.workflow.derivation.sources", {
                    sources: step.inputs.map((input) => title(input.nodeId)).join(" · "),
                  })
                }}</span>
              </UiButton>
            </li>
          </ol>
        </section>
        <section v-if="preview.inputs.length" class="mt-4">
          <h3 class="text-caption font-semibold text-fg-secondary">
            {{ t("spaces.creation.workflow.derivation.inputs") }}
          </h3>
          <ul class="mt-2 space-y-1">
            <li v-for="input in preview.inputs" :key="input.nodeId">
              <UiButton
                variant="ghost"
                class="w-full min-w-0 justify-start truncate bg-surface-control"
                @click="store.requestReveal([input.nodeId], 'ensure-visible')"
              >
                <span class="truncate">{{ title(input.nodeId) }}</span>
              </UiButton>
            </li>
          </ul>
        </section>
        <section v-if="preview.issues.length" class="mt-4">
          <h3 class="text-caption font-semibold text-fg-secondary">
            {{ t("spaces.creation.workflow.derivation.issues") }}
          </h3>
          <ul class="mt-2 space-y-1">
            <li
              v-for="issue in preview.issues"
              :key="`${issue.nodeId}:${issue.reason}`"
              class="rounded-md bg-surface-control px-2.5 py-2"
            >
              <span class="font-medium">{{ title(issue.nodeId) }}</span>
              <p class="text-caption text-fg-secondary">{{ issueText(issue.reason) }}</p>
            </li>
          </ul>
        </section>
        <p v-if="!preview.steps.length && !preview.issues.length" class="text-fg-secondary">
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
