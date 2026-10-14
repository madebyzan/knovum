<script setup lang="ts">
import { ChevronDown, RefreshCw } from "@lucide/vue";
import { storeToRefs } from "pinia";
import { computed, shallowRef } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import UiBadge from "../../../../shared/ui/UiBadge.vue";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiExpandableText from "../../../../shared/ui/UiExpandableText.vue";
import { retryEnrichmentJob } from "../../enrichment/enrichment-jobs";
import { useNodeEnrichmentStatus } from "../../enrichment/use-node-enrichment-status";
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
const visualJob = computed(() => jobs.value.find((job) => job.jobType === "analyze_visual"));
const retrying = shallowRef(false);
const tagsExpanded = shallowRef(false);
const knowledge = computed(() => {
  const data = props.node.data;
  return data.kind === "image" || data.kind === "video" ? data.visualKnowledge : undefined;
});

const visibleTags = computed(() => {
  const tags = knowledge.value?.tags ?? [];
  return tagsExpanded.value ? tags : tags.slice(0, TAG_LIMIT);
});

async function retry() {
  if (!visualJob.value || retrying.value) return;
  retrying.value = true;
  try {
    await retryEnrichmentJob(visualJob.value.id);
    await refresh();
  } catch (error) {
    toast.error(getErrorMessage(error));
  } finally {
    retrying.value = false;
  }
}
</script>

<template>
  <div>
    <template v-if="knowledge">
      <UiExpandableText
        :text="knowledge.description"
        :clamp-lines="4"
        class="text-body leading-relaxed text-fg-secondary"
      />
      <div v-if="knowledge.detectedText" class="mt-3">
        <h4 class="text-caption font-medium text-fg-tertiary">
          {{ t("spaces.inspector.visualDetectedText") }}
        </h4>
        <UiExpandableText
          :text="knowledge.detectedText"
          :clamp-lines="4"
          preformatted
          class="mt-1 text-caption leading-relaxed text-fg-secondary"
        />
      </div>
      <div v-if="knowledge.tags.length" class="mt-3 flex flex-wrap gap-1">
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
    </template>
    <p
      v-if="visualJob?.status === 'pending' || visualJob?.status === 'running'"
      class="mt-2 text-caption leading-relaxed text-fg-tertiary"
      role="status"
    >
      {{ t(`spaces.processing.enrichmentJobType.${visualJob.jobType}`) }} ·
      {{ t(`spaces.processing.jobStatus.${visualJob.status}`) }}
    </p>
    <p
      v-else-if="visualJob?.status === 'awaiting_configuration'"
      class="text-caption leading-relaxed text-fg-tertiary"
    >
      {{ t("spaces.processing.visual.waitingConfiguration") }}
    </p>
    <div
      v-else-if="visualJob?.status === 'failed'"
      class="flex items-center justify-between gap-2 text-caption text-fg-danger"
    >
      <span class="min-w-0 wrap-break-word">{{ visualJob.error ?? t("spaces.processing.failed") }}</span>
      <UiButton size="sm" variant="ghost" :disabled="retrying" @click="retry">
        <RefreshCw class="size-3" :class="retrying && 'animate-spin'" />{{ t("common.retry") }}
      </UiButton>
    </div>
    <p v-else-if="!knowledge && !loading" class="text-caption leading-relaxed text-fg-tertiary">
      {{ t("spaces.knowledge.visual.empty") }}
    </p>
  </div>
</template>
