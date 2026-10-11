<script setup lang="ts">
import { Check, ChevronRight, Circle } from "@lucide/vue";
import { computed, watch } from "vue";
import { useI18n } from "vue-i18n";

import type { ConversationMessage } from "../../contracts/generated/ConversationMessage";
import type { RunEvent } from "../../contracts/generated/RunEvent";
import { conversationActivityItems } from "../../conversation/conversation-activity-items";
import { captureActionLabel } from "../workflows/workflow-capture-labels";
import ConversationToolHistory from "./ConversationToolHistory.vue";
import SpaceConversationPlan from "./SpaceConversationPlan.vue";
import SpaceConversationShimmerText from "./SpaceConversationShimmerText.vue";
import SpaceConversationToolActivity from "./SpaceConversationToolActivity.vue";

const props = defineProps<{
  events: readonly RunEvent[];
  messages: readonly ConversationMessage[];
  runId: string;
  active: boolean;
}>();
const emit = defineEmits<{ inspect: [runId: string] }>();
watch(
  () => props.runId,
  (runId) => emit("inspect", runId),
  { immediate: true },
);
const { t } = useI18n();
const items = computed(() =>
  conversationActivityItems({
    events: props.events,
    messages: props.messages,
    runId: props.runId,
    labels: {
      action: (actionId) => captureActionLabel(actionId),
      default: t("spaces.conversation.activity.tool"),
    },
  }),
);
const toolCount = computed(() => items.value.filter((item) => item.status !== "commentary").length);
const runningTool = computed(() =>
  items.value.some((item) => item.status === "pending" || item.status === "in_progress"),
);
const unfinished = computed(
  () =>
    !props.active &&
    items.value.some(
      (item) =>
        item.status === "outcome_unknown" ||
        item.status === "failed" ||
        item.status === "pending" ||
        item.status === "in_progress",
    ),
);
</script>

<template>
  <SpaceConversationPlan :events="events" />
  <p v-if="active && !items.length" role="status" class="mt-4 py-1.5 text-caption text-fg-secondary">
    <SpaceConversationShimmerText>{{
      $t("spaces.conversation.activity.generating")
    }}</SpaceConversationShimmerText>
  </p>
  <details
    v-else-if="items.length"
    :open="active"
    class="group/activity mt-4 w-full text-caption text-fg-secondary"
  >
    <summary
      class="flex cursor-pointer list-none items-center gap-2 py-1.5 select-none hover:text-fg-primary focus-visible:outline-2 focus-visible:outline-stroke-focus"
    >
      <Circle v-if="!active && unfinished" class="size-3.5 shrink-0" />
      <Check v-else-if="!active" class="size-3.5 shrink-0" />
      <SpaceConversationShimmerText v-if="active && !runningTool">
        {{ $t("spaces.conversation.activity.generating") }}
      </SpaceConversationShimmerText>
      <SpaceConversationShimmerText v-else-if="active">
        {{ $t("spaces.conversation.activity.running", { count: toolCount }) }}
      </SpaceConversationShimmerText>
      <span v-else>{{
        $t(unfinished ? "spaces.conversation.activity.recorded" : "spaces.conversation.activity.finished", {
          count: toolCount,
        })
      }}</span>
      <ChevronRight class="size-3.5 shrink-0 transition-transform group-open/activity:rotate-90" />
    </summary>
    <div class="mt-1 space-y-1 border-l border-stroke-divider pl-3">
      <div v-for="item in items" :key="item.id" class="min-w-0 py-1">
        <p v-if="item.status === 'commentary'" class="whitespace-pre-wrap text-fg-primary">
          {{ item.title }}
        </p>
        <SpaceConversationToolActivity v-else :item="item" :active="active" />
      </div>
    </div>
  </details>
  <ConversationToolHistory :run-id="runId" :events="events" :active="active" />
</template>
