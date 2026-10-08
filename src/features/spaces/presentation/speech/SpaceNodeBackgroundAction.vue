<script setup lang="ts">
import { Music2 } from "@lucide/vue";
import { Primitive, ToolbarButton } from "reka-ui";
import { computed, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiDialog from "../../../../shared/ui/UiDialog.vue";
import UiTooltip from "../../../../shared/ui/UiTooltip.vue";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { useSpaceStore } from "../../session/space-store";
import { useSpeechMediaSources } from "../../speech/use-speech-media-sources";
import VideoSourceSeparationPanel from "./VideoSourceSeparationPanel.vue";

const props = defineProps<{ node: SpaceCanvasNode; compact?: boolean }>();
const { t } = useI18n();
const store = useSpaceStore();
const { videos, backgrounds } = useSpeechMediaSources();
const source = computed(() => [...videos.value, ...backgrounds.value].find(({ id }) => id === props.node.id));
const open = shallowRef(false);
watch([() => props.node.id, () => props.node.data.sourceRevision, () => store.activeSpaceId], () => {
  open.value = false;
});
</script>

<template>
  <component :is="compact ? ToolbarButton : Primitive" as-child>
    <UiTooltip :text="t('spaces.speech.separation.title')" :disabled="!compact" side="top">
      <UiButton
        size="sm"
        variant="ghost"
        :aria-label="t('spaces.speech.separation.title')"
        :disabled="!source || !store.activeSpaceId"
        data-testid="node-background-action"
        @click="open = true"
      >
        <Music2 class="size-3.5" :stroke-width="1.6" />
        <span v-if="!compact">{{ t("spaces.speech.separation.title") }}</span>
      </UiButton>
    </UiTooltip>
  </component>
  <UiDialog
    v-if="open && source && store.activeSpaceId"
    v-model:open="open"
    :title="t('spaces.speech.separation.title')"
    :description="node.data.title"
  >
    <VideoSourceSeparationPanel :space-id="store.activeSpaceId" :source="source.source" :disabled="false" />
  </UiDialog>
</template>
