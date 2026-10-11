<script setup lang="ts">
import { ChevronRight, Circle, Wrench } from "@lucide/vue";
import { ref } from "vue";
import { z } from "zod";

import { useSpaceStore } from "../../session/space-store";
import type { ActivityItem } from "../conversation-activity-items";
import SpaceConversationSearchMatches from "./SpaceConversationSearchMatches.vue";
import SpaceConversationShimmerText from "./SpaceConversationShimmerText.vue";
defineProps<{ item: ActivityItem; active: boolean }>();
const store = useSpaceStore();
const expanded = ref(false);
const createdNodeSchema = z.discriminatedUnion("type", [
  z.object({ type: z.literal("accepted_run"), nodeId: z.string() }),
  z.object({ type: z.literal("committed_receipt"), receipt: z.object({ nodeId: z.string().optional() }) }),
]);

function createdNodeId(value: unknown) {
  const parsed = createdNodeSchema.safeParse(value);
  return parsed.success
    ? parsed.data.type === "accepted_run"
      ? parsed.data.nodeId
      : parsed.data.receipt.nodeId
    : undefined;
}

function formatToolContent(value: unknown) {
  return typeof value === "string" ? value : JSON.stringify(value, null, 2);
}

function hasToolContent(item: ActivityItem) {
  return item.context !== undefined || item.output !== undefined;
}

function toggleTool(event: Event) {
  expanded.value = (event.target as HTMLDetailsElement).open;
}
</script>
<template>
  <component
    :is="hasToolContent(item) ? 'details' : 'div'"
    class="group/tool min-w-0"
    @toggle="toggleTool($event)"
  >
    <component
      :is="hasToolContent(item) ? 'summary' : 'div'"
      class="flex list-none items-center gap-2 rounded-md py-1"
      :class="
        hasToolContent(item)
          ? 'cursor-pointer select-none hover:text-fg-primary focus-visible:outline-2 focus-visible:outline-stroke-focus'
          : ''
      "
    >
      <Circle
        v-if="
          item.status === 'failed' ||
          item.status === 'outcome_unknown' ||
          (!active && (item.status === 'in_progress' || item.status === 'pending'))
        "
        class="size-3.5 shrink-0 text-fg-danger"
      />
      <Wrench v-else class="size-3.5 shrink-0" />
      <SpaceConversationShimmerText
        v-if="active && (item.status === 'in_progress' || item.status === 'pending')"
        class="min-w-0 truncate"
        >{{ item.title }}</SpaceConversationShimmerText
      >
      <span v-else class="min-w-0 flex-1 truncate">{{ item.title }}</span>
      <ChevronRight
        v-if="hasToolContent(item)"
        class="size-3.5 shrink-0 transition-transform group-open/tool:rotate-90"
      />
    </component>
    <div v-if="expanded && hasToolContent(item)" class="min-w-0 pb-1 pl-5 text-fg-tertiary">
      <p v-if="item.context !== undefined" class="wrap-break-word whitespace-pre-wrap">
        {{ formatToolContent(item.context) }}
      </p>
      <SpaceConversationSearchMatches v-if="item.outputName === 'search-knowledge'" :value="item.output" />
      <pre
        v-else-if="item.output !== undefined"
        class="font-mono text-caption-sm wrap-break-word whitespace-pre-wrap"
        >{{ formatToolContent(item.output) }}</pre>
      <button
        v-if="createdNodeId(item.output)"
        type="button"
        class="mt-1 text-fg-accent hover:underline focus-visible:outline-2 focus-visible:outline-stroke-focus"
        @click="store.requestReveal([createdNodeId(item.output)!], 'focus')"
      >
        {{ $t("spaces.conversation.actions.viewCreated") }}
      </button>
    </div>
  </component>
</template>
