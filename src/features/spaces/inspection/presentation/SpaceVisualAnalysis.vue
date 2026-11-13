<script setup lang="ts">
import { ChevronDown, RotateCcw } from "@lucide/vue";
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
import UiBadge from "../../../../shared/ui/UiBadge.vue";
import UiExpandableText from "../../../../shared/ui/UiExpandableText.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import { requestVisualAnalysis } from "../../enrichment/enrichment-jobs";
import SpaceEnrichmentJobStatus from "../../enrichment/presentation/SpaceEnrichmentJobStatus.vue";
import SpaceRecognizedText from "../../enrichment/presentation/SpaceRecognizedText.vue";
import { useNodeEnrichmentStatus } from "../../enrichment/use-node-enrichment-status";
import { useRecognizedText } from "../../enrichment/use-recognized-text";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { useSpaceStore } from "../../session/space-store";

const TAG_LIMIT = 8;

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
const tagsExpanded = shallowRef(false);
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

const visibleTags = computed(() => {
  const tags = knowledge.value?.tags ?? [];
  return tagsExpanded.value ? tags : tags.slice(0, TAG_LIMIT);
});

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
  <div>
    <div class="mb-1.5 flex items-center justify-between gap-2">
      <h3 class="text-label font-medium text-fg-primary">
        {{ t("spaces.inspector.content") }}
      </h3>
      <ModelChoiceTrigger
        v-if="canReanalyze"
        :options="visualTargets"
        :disabled="submitting"
        @select="(id) => void reanalyze(id)"
      >
        <template #trigger="{ multiple }">
          <UiIconButton
            :label="t('spaces.processing.visual.reanalyze')"
            size="sm"
            :disabled="submitting"
            @click="!multiple && reanalyze()"
          >
            <RotateCcw class="size-3" />
          </UiIconButton>
        </template>
      </ModelChoiceTrigger>
    </div>
    <div v-if="knowledge" class="flex items-start gap-2">
      <UiExpandableText
        :text="knowledge.description"
        :clamp-lines="4"
        class="min-w-0 flex-1 text-body leading-relaxed text-fg-secondary"
      />
    </div>
    <SpaceEnrichmentJobStatus :job="visualJob" @retried="refresh" />
    <p v-if="!visualJob && !knowledge && !loading" class="text-caption leading-relaxed text-fg-tertiary">
      {{ t("spaces.knowledge.visual.empty") }}
    </p>
    <SpaceRecognizedText class="mt-3" :text="recognizedText(null)" :job="recognitionJob" @retried="refresh" />
    <div v-if="knowledge?.tags.length" class="mt-3 space-y-1">
      <h4 class="text-caption font-medium text-fg-tertiary">
        {{ t("spaces.tags.title") }}
      </h4>
      <div class="flex flex-wrap gap-1">
        <UiBadge v-for="tag in visibleTags" :key="tag" size="sm">
          {{ tag }}
        </UiBadge>
        <button
          v-if="knowledge.tags.length > TAG_LIMIT"
          type="button"
          :aria-expanded="tagsExpanded"
          class="inline-flex h-4 cursor-pointer items-center gap-0.5 rounded-md bg-surface-control px-1.5 text-caption-sm font-medium text-fg-secondary transition-colors duration-150 select-none hover:bg-surface-control-hover hover:text-fg-primary focus-visible:outline-2 focus-visible:outline-stroke-focus"
          @click="tagsExpanded = !tagsExpanded"
        >
          <span>{{
            tagsExpanded ? t("common.collapse") : `+${knowledge.tags.length - TAG_LIMIT} ${t("common.more")}`
          }}</span>
          <ChevronDown
            class="size-2.5 text-fg-tertiary transition-transform duration-150"
            :class="tagsExpanded && 'rotate-180'"
          />
        </button>
      </div>
    </div>
  </div>
</template>
