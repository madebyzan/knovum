<script setup lang="ts">
import { Clock3, RefreshCw } from "@lucide/vue";
import { storeToRefs } from "pinia";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import { useSpaceKnowledgeHistoryQuery } from "../../knowledge/queries/queries";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { useSpaceStore } from "../../session/space-store";
import SpaceInspectorSection from "../SpaceInspectorSection.vue";

const props = defineProps<{ node: SpaceCanvasNode }>();
const { t } = useI18n();
const { activeSpaceId } = storeToRefs(useSpaceStore());
const history = useSpaceKnowledgeHistoryQuery(activeSpaceId, () => props.node.id);
</script>

<template>
  <SpaceInspectorSection
    :title="t('spaces.knowledge.knowledgeHistory')"
    :icon="Clock3"
    :default-expanded="false"
  >
    <div v-if="history.isError.value" class="rounded-control border border-danger/30 bg-danger/5 p-2">
      <p class="text-micro text-danger">{{ t("spaces.knowledge.knowledgeHistoryLoadFailed") }}</p>
      <UiButton
        class="mt-1"
        size="sm"
        variant="ghost"
        :disabled="history.isFetching.value"
        @click="history.refetch()"
      >
        <RefreshCw class="size-(--knovum-icon-size-xs)" />{{ t("common.retry") }}
      </UiButton>
    </div>
    <div v-else-if="history.data.value?.length" class="space-y-1">
      <div
        v-for="event in history.data.value"
        :key="event.id"
        class="rounded-control bg-control/35 px-2 py-1.5"
      >
        <div class="flex items-center justify-between gap-2">
          <span class="text-caption text-foreground-secondary">{{
            t(`spaces.knowledge.knowledgeEvent.${event.eventKind}`)
          }}</span>
          <time class="text-micro text-foreground-tertiary">{{
            new Date(event.createdAt).toLocaleString()
          }}</time>
        </div>
        <p class="mt-0.5 text-micro text-foreground-tertiary">
          {{ t("spaces.knowledge.knowledgeActor", { actor: event.actorKind }) }}
        </p>
      </div>
    </div>
    <p v-else-if="!history.isFetching.value" class="px-1 text-caption text-foreground-tertiary">
      {{ t("spaces.knowledge.knowledgeHistoryEmpty") }}
    </p>
  </SpaceInspectorSection>
</template>
