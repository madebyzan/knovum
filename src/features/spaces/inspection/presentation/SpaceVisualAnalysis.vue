<script setup lang="ts">
import { RotateCcw, ScanEye } from "@lucide/vue";
import { storeToRefs } from "pinia";
import { computed, shallowRef } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { admitAiRequirements } from "../../../../foundation/ai/ai-readiness-admission";
import ModelChoiceTrigger from "../../../../foundation/ai/ModelChoiceTrigger.vue";
import { useRoutedTargetOptions } from "../../../../foundation/ai/use-model-choice-options";
import { getErrorMessage } from "../../../../foundation/errors/error-format";
import type { TaskTarget } from "../../../../shared/contracts/generated/TaskTarget";
import type { TaskTargetSelection } from "../../../../shared/contracts/generated/TaskTargetSelection";
import UiExpandableText from "../../../../shared/ui/UiExpandableText.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import { requestVisualAnalysis } from "../../enrichment/enrichment-jobs";
import SpaceEnrichmentJobStatus from "../../enrichment/presentation/SpaceEnrichmentJobStatus.vue";
import SpaceRecognizedText from "../../enrichment/presentation/SpaceRecognizedText.vue";
import { useNodeEnrichmentStatus } from "../../enrichment/use-node-enrichment-status";
import { useRecognizedText } from "../../enrichment/use-recognized-text";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { useSpaceStore } from "../../session/space-store";
import SpaceInspectorSection from "./SpaceInspectorSection.vue";
import SpaceVisualKeywords from "./SpaceVisualKeywords.vue";

const props = defineProps<{ node: SpaceCanvasNode }>();
const { t } = useI18n();
const { activeSpaceId } = storeToRefs(useSpaceStore());
const { jobs, loading, refresh } = useNodeEnrichmentStatus(
  activeSpaceId,
  computed(() => props.node.id),
);
const submitting = shallowRef(false);
const visualJob = computed(() => jobs.value.find((job) => job.jobType === "analyze_visual"));
const {
  options: visualTargets,
  defaultTarget: defaultVisualTarget,
  targetForId: visualTargetForId,
} = useRoutedTargetOptions(() => "visual-understanding");
const recognitionJob = computed(() => jobs.value.find((job) => job.jobType === "recognize_text"));
const recognizedText = useRecognizedText(
  activeSpaceId,
  computed(() => props.node.id),
  computed(() => {
    const data = props.node.data;
    return data.kind === "image" || data.kind === "video" ? data.textRecognition?.sourceRevision : undefined;
  }),
);
const knowledge = computed(() => {
  const data = props.node.data;
  return data.kind === "image" || data.kind === "video" ? data.visualKnowledge : undefined;
});
const canRequestVisual = computed(() => {
  const data = props.node.data;
  if (data.kind === "image") return Boolean(data.sourceRevision);
  if (data.kind === "video") return Boolean(data.sourceRevision && data.previewPath);
  return false;
});
const canReanalyze = computed(
  () => canRequestVisual.value && (!visualJob.value || visualJob.value.status === "awaiting_configuration"),
);

function targetFromSelection(selection: TaskTargetSelection | null | undefined): TaskTarget | undefined {
  if (!selection) return undefined;
  return selection.type === "model"
    ? { type: "model", connectionId: selection.connectionId, modelId: selection.modelId }
    : { type: "agent", runtimeKind: selection.runtimeKind };
}

async function reanalyze(targetId?: string) {
  if (!activeSpaceId.value || submitting.value || !canReanalyze.value) return;
  const target = targetId
    ? visualTargetForId(targetId)
    : (defaultVisualTarget.value ?? targetFromSelection(visualJob.value?.executionTarget));
  if (!target) {
    await admitAiRequirements([{ kind: "routed", purpose: "visual-understanding" }]);
    return;
  }
  submitting.value = true;
  try {
    await requestVisualAnalysis(activeSpaceId.value, props.node.id, target);
    await refresh();
  } catch (error) {
    toast.error(getErrorMessage(error));
  } finally {
    submitting.value = false;
  }
}
</script>

<template>
  <SpaceInspectorSection :title="t('spaces.inspector.content')" :icon="ScanEye">
    <template v-if="canReanalyze" #actions>
      <ModelChoiceTrigger
        :options="visualTargets"
        :disabled="submitting"
        @select="(id) => void reanalyze(id)"
      >
        <template #trigger="{ multiple }">
          <UiIconButton
            :label="t('spaces.processing.visual.reanalyze')"
            tooltip
            size="sm"
            :disabled="submitting"
            @click="!multiple && reanalyze()"
          >
            <RotateCcw class="size-3" />
          </UiIconButton>
        </template>
      </ModelChoiceTrigger>
    </template>
    <UiExpandableText
      v-if="knowledge"
      :text="knowledge.description"
      :clamp-lines="4"
      class="text-body leading-relaxed text-fg-secondary"
    />
    <SpaceEnrichmentJobStatus :job="visualJob" @retried="refresh" />
    <p v-if="!visualJob && !knowledge && !loading" class="text-caption leading-relaxed text-fg-tertiary">
      {{ t("spaces.knowledge.visual.empty") }}
    </p>
    <SpaceRecognizedText class="mt-3" :text="recognizedText(null)" :job="recognitionJob" @retried="refresh" />
    <SpaceVisualKeywords v-if="knowledge?.tags.length" class="mt-3" :node="node" :keywords="knowledge.tags" />
  </SpaceInspectorSection>
</template>
