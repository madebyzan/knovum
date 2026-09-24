<script setup lang="ts">
import { CircleAlert } from "@lucide/vue";
import { storeToRefs } from "pinia";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import { useImageEditSession } from "../../../image-editing/image-edit-session";
import type { SpaceNodePayload } from "../../../model/nodes/node-payload";
import { useSpaceStore } from "../../../session/space-store";
import { canvasNodeWork } from "./canvas-node-work";
import SpaceCanvasWorkDots from "./SpaceCanvasWorkDots.vue";

const props = defineProps<{ id: string; data: SpaceNodePayload }>();
const { t } = useI18n();
const { mediaAnalysisIds } = storeToRefs(useSpaceStore());
const imageEditSession = useImageEditSession();
const work = computed(() =>
  canvasNodeWork(props.data, {
    mediaAnalysis: mediaAnalysisIds.value.includes(props.id),
    imageEditApplying: imageEditSession.applying && imageEditSession.active?.nodeId === props.id,
  }),
);
const label = computed(() => (work.value ? t(work.value.labelKey) : ""));
</script>

<template>
  <span v-if="work?.surface === 'chrome'" :title="label" :aria-label="label" role="status">
    <CircleAlert
      v-if="work.phase === 'failed'"
      class="size-(--knovum-icon-size-xs) text-danger"
      :stroke-width="1.6"
    />
    <SpaceCanvasWorkDots v-else placement="inline" />
  </span>
  <span v-else-if="work?.surface === 'selection'" class="text-micro text-accent" role="status">
    {{ label }}
  </span>
</template>
