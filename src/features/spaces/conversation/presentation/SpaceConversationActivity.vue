<script setup lang="ts">
import { Check, ChevronRight, Circle } from "@lucide/vue";
import { computed } from "vue";

import type { ActivityItem } from "../conversation-activity-items";
import SpaceConversationShimmerText from "./SpaceConversationShimmerText.vue";
import SpaceConversationToolActivity from "./SpaceConversationToolActivity.vue";

const props = defineProps<{
  items: readonly ActivityItem[];
  /** The run is producing: a call still under way is in progress, not left unfinished. */
  active: boolean;
  /** The run is working here now — nothing has been written since these calls. */
  current: boolean;
}>();
// The group is closed until the person opens it, and then only they close it. It never opens
// or closes by itself: a list that unfolds for each call and folds when the call ends moves
// everything below it twice per tool.
const unfinished = computed(() =>
  props.items.some(
    (item) =>
      item.status === "outcome_unknown" ||
      item.status === "failed" ||
      (!props.active && (item.status === "pending" || item.status === "in_progress")),
  ),
);
</script>

<template>
  <details class="group/activity w-full text-caption text-fg-secondary">
    <summary
      class="flex w-fit max-w-full cursor-pointer list-none items-center gap-1.5 rounded-md py-1 select-none hover:text-fg-primary focus-visible:outline-2 focus-visible:outline-stroke-focus [&::-webkit-details-marker]:hidden"
    >
      <SpaceConversationShimmerText v-if="current" indicator>
        {{ $t("spaces.conversation.activity.running", { count: items.length }) }}
      </SpaceConversationShimmerText>
      <template v-else>
        <Circle v-if="unfinished" class="size-3.5 shrink-0 text-fg-warning" />
        <Check v-else class="size-3.5 shrink-0 text-fg-tertiary" />
        <span class="truncate">{{
          $t(unfinished ? "spaces.conversation.activity.recorded" : "spaces.conversation.activity.finished", {
            count: items.length,
          })
        }}</span>
      </template>
      <ChevronRight
        class="size-3.5 shrink-0 text-fg-tertiary transition-transform duration-150 group-open/activity:rotate-90"
      />
    </summary>
    <div class="mt-1 space-y-1 border-l border-stroke-divider pl-3">
      <SpaceConversationToolActivity
        v-for="item in items"
        :key="item.id"
        class="py-1"
        :item="item"
        :active="active"
      />
    </div>
  </details>
</template>
