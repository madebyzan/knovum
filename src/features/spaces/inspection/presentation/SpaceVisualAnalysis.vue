<script setup lang="ts">
import { ChevronDown } from "@lucide/vue";
import { storeToRefs } from "pinia";
import { computed, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import UiBadge from "../../../../shared/ui/UiBadge.vue";
import UiExpandableText from "../../../../shared/ui/UiExpandableText.vue";
import { useNodeEnrichmentStatus } from "../../enrichment/use-node-enrichment-status";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { useSpaceStore } from "../../session/space-store";
import SpaceEnrichmentJobStatus from "./SpaceEnrichmentJobStatus.vue";

const TAG_LIMIT = 8;

const props = defineProps<{ node: SpaceCanvasNode }>();
const { t } = useI18n();
const { activeSpaceId } = storeToRefs(useSpaceStore());
const { jobs, loading, refresh } = useNodeEnrichmentStatus(
  activeSpaceId,
  computed(() => props.node.id),
);
const visualJob = computed(() => jobs.value.find((job) => job.jobType === "analyze_visual"));
const tagsExpanded = shallowRef(false);
const knowledge = computed(() => {
  const data = props.node.data;
  return data.kind === "image" || data.kind === "video" ? data.visualKnowledge : undefined;
});

const visibleTags = computed(() => {
  const tags = knowledge.value?.tags ?? [];
  return tagsExpanded.value ? tags : tags.slice(0, TAG_LIMIT);
});
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
    <SpaceEnrichmentJobStatus :job="visualJob" @retried="refresh" />
    <p v-if="!visualJob && !knowledge && !loading" class="text-caption leading-relaxed text-fg-tertiary">
      {{ t("spaces.knowledge.visual.empty") }}
    </p>
  </div>
</template>
