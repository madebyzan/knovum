<script setup lang="ts">
import { ScanText } from "@lucide/vue";
import { storeToRefs } from "pinia";
import { computed } from "vue";

import UiExpandableText from "../../../../shared/ui/UiExpandableText.vue";
import { useNodeEnrichmentStatus } from "../../enrichment/use-node-enrichment-status";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { useSpaceStore } from "../../session/space-store";
import SpaceEnrichmentJobStatus from "./SpaceEnrichmentJobStatus.vue";
import SpaceInspectorSection from "./SpaceInspectorSection.vue";

const props = defineProps<{ node: SpaceCanvasNode }>();
const { activeSpaceId } = storeToRefs(useSpaceStore());
const { jobs, refresh } = useNodeEnrichmentStatus(
  activeSpaceId,
  computed(() => props.node.id),
);
const job = computed(() => jobs.value.find(({ jobType }) => jobType === "recognize_text"));
const text = computed(() => {
  const { data } = props.node;
  return data.kind === "image" || data.kind === "file" ? data.textRecognition?.text : undefined;
});
</script>

<template>
  <SpaceInspectorSection
    v-if="text || job"
    :title="$t('spaces.inspector.recognizedText')"
    :icon="ScanText"
    :default-expanded="false"
  >
    <UiExpandableText
      v-if="text"
      :text="text"
      :clamp-lines="6"
      preformatted
      class="text-caption leading-relaxed text-fg-secondary"
    />
    <SpaceEnrichmentJobStatus :job="job" @retried="refresh" />
  </SpaceInspectorSection>
</template>
