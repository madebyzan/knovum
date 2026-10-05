<script setup lang="ts">
import { Check, ChevronRight, Circle, Wrench } from "@lucide/vue";
import { computed, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";
import { z } from "zod";

import type { AssistantMessage } from "../../contracts/generated/AssistantMessage";
import type { RunEvent } from "../../contracts/generated/RunEvent";
import { conversationActivityItems, type ActivityItem } from "../../conversation/conversation-activity-items";
import { useSpaceStore } from "../../session/space-store";
import { captureActionLabel } from "../workflows/workflow-capture-labels";
import SpaceConversationSearchMatches from "./SpaceConversationSearchMatches.vue";
import SpaceConversationShimmerText from "./SpaceConversationShimmerText.vue";

const props = defineProps<{
  events: readonly RunEvent[];
  messages: readonly AssistantMessage[];
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
const store = useSpaceStore();
const expanded = shallowRef(new Set<string>());
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

const items = computed(() =>
  conversationActivityItems({
    events: props.events,
    messages: props.messages,
    runId: props.runId,
    labels: {
      action: (id) => captureActionLabel(id, t),
      default: t("spaces.assistant.activity.tool"),
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
      (item) => item.status === "failed" || item.status === "pending" || item.status === "in_progress",
    ),
);

function formatToolContent(value: unknown) {
  return typeof value === "string" ? value : JSON.stringify(value, null, 2);
}

function hasToolContent(item: ActivityItem) {
  return item.context !== undefined || item.output !== undefined;
}

function toggleTool(event: Event, id: string) {
  const next = new Set(expanded.value);
  if ((event.target as HTMLDetailsElement).open) next.add(id);
  else next.delete(id);
  expanded.value = next;
}
</script>

<template>
  <p v-if="active && !items.length" role="status" class="mt-4 py-1.5 text-caption text-fg-secondary">
    <SpaceConversationShimmerText>{{
      $t("spaces.assistant.activity.generating")
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
        {{ $t("spaces.assistant.activity.generating") }}
      </SpaceConversationShimmerText>
      <SpaceConversationShimmerText v-else-if="active">
        {{ $t("spaces.assistant.activity.running", { count: toolCount }) }}
      </SpaceConversationShimmerText>
      <span v-else>{{
        $t(unfinished ? "spaces.assistant.activity.recorded" : "spaces.assistant.activity.finished", {
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
        <component
          :is="hasToolContent(item) ? 'details' : 'div'"
          v-else
          class="group/tool min-w-0"
          @toggle="toggleTool($event, item.id)"
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
          <div
            v-if="expanded.has(item.id) && hasToolContent(item)"
            class="min-w-0 pb-1 pl-5 text-fg-tertiary"
          >
            <p v-if="item.context !== undefined" class="wrap-break-word whitespace-pre-wrap">
              {{ formatToolContent(item.context) }}
            </p>
            <SpaceConversationSearchMatches
              v-if="item.outputName === 'search-knowledge'"
              :value="item.output"
            />
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
              {{ $t("spaces.assistant.actions.viewCreated") }}
            </button>
          </div>
        </component>
      </div>
    </div>
  </details>
</template>
