<script setup lang="ts">
import { CircleAlert, LoaderCircle, RefreshCw } from "@lucide/vue";
import { onBeforeUnmount, ref, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import {
  knowledgeIndexStatus,
  scheduleSpaceKnowledgeIndex,
  subscribeKnowledgeIndexStatus,
  type KnowledgeIndexStatus,
} from "../../knowledge/indexing/index-scheduler";

const props = defineProps<{ spaceId?: string }>();
const { t } = useI18n();
const status = ref<KnowledgeIndexStatus>("idle");
let unsubscribe: (() => void) | undefined;

watch(() => props.spaceId, bind, { immediate: true });
onBeforeUnmount(() => unsubscribe?.());

function bind(spaceId?: string) {
  unsubscribe?.();
  unsubscribe = undefined;
  status.value = spaceId ? knowledgeIndexStatus(spaceId) : "idle";
  if (spaceId)
    unsubscribe = subscribeKnowledgeIndexStatus(spaceId, (value) => {
      status.value = value;
    });
}
function retry() {
  if (props.spaceId) scheduleSpaceKnowledgeIndex(props.spaceId);
}
</script>

<template>
  <div
    v-if="status === 'scheduled' || status === 'indexing' || status === 'unavailable'"
    class="flex min-h-(--knovum-control-height-sm) items-center gap-1.5 px-1 text-micro text-foreground-tertiary"
  >
    <LoaderCircle
      v-if="status === 'scheduled' || status === 'indexing'"
      class="size-(--knovum-icon-size-xs) animate-spin"
    />
    <CircleAlert v-else-if="status === 'unavailable'" class="size-(--knovum-icon-size-xs) text-warning" />
    <span class="min-w-0 flex-1">
      {{ t(`spaces.knowledge.knowledgeIndex.${status}`) }}
    </span>
    <UiButton v-if="status === 'unavailable'" size="sm" variant="ghost" @click="retry">
      <RefreshCw class="size-(--knovum-icon-size-xs)" />{{ t("common.retry") }}
    </UiButton>
  </div>
</template>
