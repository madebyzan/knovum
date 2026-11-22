<script setup lang="ts">
import { Clock3, RefreshCw } from "@lucide/vue";
import { storeToRefs } from "pinia";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import SpaceInspectorSection from "../../inspection/presentation/SpaceInspectorSection.vue";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { useSpaceStore } from "../../session/space-store";
import { useSpaceKnowledgeHistoryQuery } from "../queries/use-space-knowledge-history-query";

const props = defineProps<{ node: SpaceCanvasNode }>();
const { d, t } = useI18n();
const { activeSpaceId } = storeToRefs(useSpaceStore());
const history = useSpaceKnowledgeHistoryQuery(activeSpaceId, () => props.node.id);
</script>

<template>
  <SpaceInspectorSection
    :title="t('spaces.knowledge.history.title')"
    :icon="Clock3"
    :count="history.data.value?.length"
    :default-expanded="false"
  >
    <div
      v-if="history.isError.value"
      class="rounded-lg border border-stroke-danger/30 bg-surface-danger-subtle px-2.5 py-2"
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
    <ol
      v-else-if="history.data.value?.length"
      class="ml-1.5 space-y-2 border-l border-stroke-divider/70 pl-3"
    >
      <li v-for="event in history.data.value" :key="event.id" class="relative">
        <span
          aria-hidden="true"
          class="absolute top-1.5 -left-[15px] size-1.5 rounded-full bg-fg-tertiary/60"
        />
        <div class="flex items-baseline justify-between gap-2">
          <span class="text-caption font-medium text-fg-secondary">{{
            t(`spaces.knowledge.history.events.${event.eventKind}`)
          }}</span>
          <time class="shrink-0 text-caption-sm text-fg-tertiary tabular-nums">{{
            d(event.createdAt, "short")
          }}</time>
        </div>
        <p class="text-caption-sm text-fg-tertiary">
          {{ t("spaces.knowledge.history.actor", { actor: event.actorKind }) }}
        </p>
      </li>
    </ol>
    <p v-else-if="!history.isFetching.value" class="text-caption text-fg-tertiary">
      {{ t("spaces.knowledge.history.empty") }}
    </p>
  </SpaceInspectorSection>
</template>
