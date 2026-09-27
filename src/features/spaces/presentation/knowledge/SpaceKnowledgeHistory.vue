<script setup lang="ts">
import { Clock3, RefreshCw } from "@lucide/vue";
import { storeToRefs } from "pinia";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import { useSpaceKnowledgeHistoryQuery } from "../../knowledge/queries/use-knowledge-queries";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { useSpaceStore } from "../../session/space-store";
import SpaceInspectorSection from "../inspector/SpaceInspectorSection.vue";

const props = defineProps<{ node: SpaceCanvasNode }>();
const { d, t } = useI18n();
const { activeSpaceId } = storeToRefs(useSpaceStore());
const history = useSpaceKnowledgeHistoryQuery(activeSpaceId, () => props.node.id);
</script>

<template>
  <SpaceInspectorSection
    :title="t('spaces.knowledge.history.title')"
    :icon="Clock3"
    :default-expanded="false"
  >
    <div
      v-if="history.isError.value"
      class="rounded-md border border-stroke-danger/30 bg-surface-danger/5 p-2"
    >
      <p class="text-caption-sm text-fg-danger">{{ t("spaces.knowledge.history.loadFailed") }}</p>
      <UiButton
        class="mt-1"
        size="sm"
        variant="ghost"
        :disabled="history.isFetching.value"
        @click="history.refetch()"
      >
        <RefreshCw class="size-3" />{{ t("common.retry") }}
      </UiButton>
    </div>
    <div v-else-if="history.data.value?.length" class="space-y-1">
      <div
        v-for="event in history.data.value"
        :key="event.id"
        class="rounded-md bg-surface-control/35 px-2 py-1.5"
      >
        <div class="flex items-center justify-between gap-2">
          <span class="text-caption text-fg-secondary">{{
            t(`spaces.knowledge.history.events.${event.eventKind}`)
          }}</span>
          <time class="text-caption-sm text-fg-tertiary">{{ d(event.createdAt, "short") }}</time>
        </div>
        <p class="mt-0.5 text-caption-sm text-fg-tertiary">
          {{ t("spaces.knowledge.history.actor", { actor: event.actorKind }) }}
        </p>
      </div>
    </div>
    <p v-else-if="!history.isFetching.value" class="px-1 text-caption text-fg-tertiary">
      {{ t("spaces.knowledge.history.empty") }}
    </p>
  </SpaceInspectorSection>
</template>
