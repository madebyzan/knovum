<script setup lang="ts">
import { Check, ChevronRight, Circle } from "@lucide/vue";
import { computed, shallowRef, watch } from "vue";

import type { ActivityItem } from "../conversation-activity-items";
import SpaceConversationShimmerText from "./SpaceConversationShimmerText.vue";
import SpaceConversationToolActivity from "./SpaceConversationToolActivity.vue";

const props = defineProps<{ items: readonly ActivityItem[]; active: boolean }>();
const toolCount = computed(() => props.items.filter((item) => item.status !== "commentary").length);
const runningTool = computed(() =>
  props.items.some((item) => item.status === "pending" || item.status === "in_progress"),
);
const unfinished = computed(() =>
  props.items.some(
    (item) =>
      item.status === "outcome_unknown" ||
      item.status === "failed" ||
      (!props.active && (item.status === "pending" || item.status === "in_progress")),
  ),
);
const userOpen = shallowRef<boolean>();
const isOpen = computed(() => userOpen.value ?? (props.active && runningTool.value));

watch(
  () => props.active && runningTool.value,
  (running) => {
    if (!running) userOpen.value = undefined;
  },
);

function toggleActivity(event: Event) {
  const details = event.currentTarget as HTMLDetailsElement | null;
  if (!details) return;
  const defaultOpen = props.active && runningTool.value;
  userOpen.value = details.open === defaultOpen ? undefined : details.open;
}
</script>

<template>
  <details
    :open="isOpen"
    class="group/activity w-full text-caption text-fg-secondary"
    @toggle="toggleActivity"
  >
    <summary
      class="flex w-fit max-w-full cursor-pointer list-none items-center gap-1.5 rounded-md py-1 select-none hover:text-fg-primary focus-visible:outline-2 focus-visible:outline-stroke-focus [&::-webkit-details-marker]:hidden"
    >
      <SpaceConversationShimmerText v-if="active && runningTool" indicator>
        {{ $t("spaces.conversation.activity.running", { count: toolCount }) }}
      </SpaceConversationShimmerText>
      <template v-else>
        <Circle v-if="unfinished" class="size-3.5 shrink-0 text-fg-warning" />
        <Check v-else class="size-3.5 shrink-0 text-fg-tertiary" />
        <span class="truncate">{{
          $t(unfinished ? "spaces.conversation.activity.recorded" : "spaces.conversation.activity.finished", {
            count: toolCount,
          })
        }}</span>
      </template>
      <ChevronRight
        class="size-3.5 shrink-0 text-fg-tertiary transition-transform duration-150 group-open/activity:rotate-90"
      />
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
