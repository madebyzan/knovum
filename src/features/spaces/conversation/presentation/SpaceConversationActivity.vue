<script setup lang="ts">
import { Check, ChevronRight, Circle } from "@lucide/vue";
import { computed } from "vue";

import type { ActivityItem } from "../conversation-activity-items";
import SpaceConversationShimmerText from "./SpaceConversationShimmerText.vue";
import SpaceConversationToolActivity from "./SpaceConversationToolActivity.vue";

const props = defineProps<{ items: readonly ActivityItem[]; active: boolean }>();
const toolCount = computed(() => props.items.filter((item) => item.status !== "commentary").length);
const runningTool = computed(() =>
  props.items.some((item) => item.status === "pending" || item.status === "in_progress"),
);
const unfinished = computed(
  () =>
    !props.active &&
    props.items.some(
      (item) =>
        item.status === "outcome_unknown" ||
        item.status === "failed" ||
        item.status === "pending" ||
        item.status === "in_progress",
    ),
);
</script>

<template>
  <details :open="active" class="group/activity w-full text-caption text-fg-secondary">
    <summary
      class="flex cursor-pointer list-none items-center gap-1.5 rounded-md py-1 select-none hover:text-fg-primary focus-visible:outline-2 focus-visible:outline-stroke-focus"
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
</template>
