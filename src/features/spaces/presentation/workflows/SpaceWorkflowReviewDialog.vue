<script setup lang="ts">
import { computed, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import { decodeCommandError } from "../../../../foundation/ipc/command-error";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiDialog from "../../../../shared/ui/UiDialog.vue";
import UiMarkdown from "../../../../shared/ui/UiMarkdown.vue";
import type { StepSummary } from "../../contracts/generated/RunStep";
import type { SpaceNodePayload } from "../../model/nodes/node-payload";
import { useSpaceStore } from "../../session/space-store";
import { decideWorkflowReview, loadWorkflowReview } from "../../workflows/workflow-review";

const props = defineProps<{ runId: string }>();
const emit = defineEmits<{ close: [] }>();
const { t } = useI18n();
const space = useSpaceStore();
const open = computed({ get: () => true, set: (value: boolean) => !value && emit("close") });
const loading = shallowRef(false);
const deciding = shallowRef(false);
const error = shallowRef("");
const step = shallowRef<StepSummary>();
const content = shallowRef<SpaceNodePayload>();
const reviewedRevision = shallowRef("");
const reviewedNodeId = shallowRef("");
const edited = shallowRef(false);
const contentTitle = computed(() => content.value?.title ?? "");
const subtitleText = computed(() =>
  content.value?.kind === "subtitle" ? content.value.cues.map((cue) => cue.text).join("\n") : "",
);

async function load() {
  const spaceId = space.activeSpaceId;
  if (!spaceId) return;
  loading.value = true;
  error.value = "";
  step.value = undefined;
  content.value = undefined;
  reviewedRevision.value = "";
  reviewedNodeId.value = "";
  try {
    const review = await loadWorkflowReview(spaceId, props.runId);
    step.value = review.step;
    content.value = review.content;
    reviewedRevision.value = review.revision;
    reviewedNodeId.value = review.nodeId;
    edited.value = review.edited;
  } catch (cause) {
    error.value =
      decodeCommandError(cause)?.code === "workflow_review_unavailable"
        ? t("spaces.creation.workflow.review.unavailable")
        : String(cause);
  } finally {
    loading.value = false;
  }
}

async function decide(approved: boolean) {
  if (!step.value) return;
  deciding.value = true;
  error.value = "";
  try {
    await decideWorkflowReview({
      stepId: step.value.id,
      expectedVersion: step.value.stateVersion,
      reviewedRevision: reviewedRevision.value,
      approved,
    });
    emit("close");
  } catch (cause) {
    await load();
    error.value =
      decodeCommandError(cause)?.code === "reviewed_revision_changed"
        ? t("spaces.creation.workflow.review.changed")
        : String(cause);
  } finally {
    deciding.value = false;
  }
}

function editOnCanvas() {
  if (!reviewedNodeId.value) return;
  space.requestReveal([reviewedNodeId.value], "focus");
  if (content.value?.kind === "markdown") space.focusedEditorId = reviewedNodeId.value;
  emit("close");
}

watch(
  () => props.runId,
  () => void load(),
  { immediate: true },
);
</script>

<template>
  <UiDialog v-model:open="open" :title="t('spaces.creation.workflow.review.title')" size="wide">
    <p v-if="loading" class="text-caption text-foreground-secondary">{{ t("common.loading") }}</p>
    <template v-else-if="content && step">
      <h3 class="mb-3 text-body font-medium">{{ contentTitle }}</h3>
      <p v-if="edited" class="mb-3 text-caption text-foreground-secondary">
        {{ t("spaces.creation.workflow.review.edited") }}
      </p>
      <UiMarkdown v-if="content.kind === 'markdown'" :content="content.text" />
      <p v-else-if="content.kind === 'subtitle'" class="text-body leading-relaxed whitespace-pre-wrap">
        {{ subtitleText }}
      </p>
    </template>
    <p v-if="error" role="alert" class="mt-3 text-caption text-danger">{{ error }}</p>
    <template #footer>
      <UiButton variant="ghost" :disabled="!step || deciding" @click="editOnCanvas">{{
        t("spaces.creation.workflow.review.edit")
      }}</UiButton>
      <UiButton :disabled="deciding || !step" @click="decide(false)">{{
        t("spaces.creation.workflow.review.reject")
      }}</UiButton>
      <UiButton variant="primary" :disabled="deciding || !step" @click="decide(true)">{{
        t("spaces.creation.workflow.review.approve")
      }}</UiButton>
    </template>
  </UiDialog>
</template>
