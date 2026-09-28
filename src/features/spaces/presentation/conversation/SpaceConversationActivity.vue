<script setup lang="ts">
import { Check, ChevronRight, Circle, LoaderCircle, Shield, Wrench } from "@lucide/vue";
import { computed, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";
import { z } from "zod";

import type { RunEvent } from "../../contracts/generated/RunEvent";
import { conversationActivityItems, type ActivityItem } from "../../conversation/conversation-activity-items";
import type { AssistantMessage } from "../../conversation/conversation-ipc";
import { useSpaceStore } from "../../session/space-store";
import { captureActionLabel } from "../workflows/workflow-capture-labels";
import SpaceConversationSearchMatches from "./SpaceConversationSearchMatches.vue";

const props = defineProps<{
  events: readonly RunEvent[];
  messages: readonly AssistantMessage[];
  runId: string;
  active: boolean;
}>();
const emit = defineEmits<{ inspect: [runId: string] }>();
watch(() => props.runId, (runId) => emit("inspect", runId), { immediate: true });
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
    ? parsed.data.type === "accepted_run" ? parsed.data.nodeId : parsed.data.receipt.nodeId
    : undefined;
}

const items = computed(() => conversationActivityItems(
  props.events, props.messages, props.runId,
  (id) => captureActionLabel(id, t),
  t("spaces.assistant.activity.tool"),
));
const toolCount = computed(() => items.value.filter((item) => item.status !== "commentary").length);
const unfinished = computed(() => !props.active && items.value.some((item) =>
  item.status === "failed" || item.status === "pending" || item.status === "in_progress",
));

function resultText(value: unknown) {
  return typeof value === "string" ? value : JSON.stringify(value, null, 2);
}

function hasDetails(item: ActivityItem) {
  return item.detail !== undefined || item.result !== undefined;
}

function toggleTool(event: Event, id: string) {
  const next = new Set(expanded.value);
  if ((event.target as HTMLDetailsElement).open) next.add(id);
  else next.delete(id);
  expanded.value = next;
}
</script>

<template>
  <details v-if="items.length || active" :open="active" class="group/activity mt-4 w-full text-caption text-fg-secondary">
    <summary class="flex cursor-pointer list-none items-center gap-2 py-1.5 select-none hover:text-fg-primary focus-visible:outline-2 focus-visible:outline-stroke-focus">
      <LoaderCircle v-if="active" class="size-3.5 shrink-0 animate-spin text-fg-accent" />
      <Circle v-else-if="unfinished" class="size-3.5 shrink-0" />
      <Check v-else class="size-3.5 shrink-0" />
      <span>{{ $t(active ? "spaces.assistant.activity.running" : unfinished ? "spaces.assistant.activity.recorded" : "spaces.assistant.activity.finished", { count: toolCount }) }}</span>
      <ChevronRight class="size-3.5 shrink-0 transition-transform group-open/activity:rotate-90" />
    </summary>
    <div class="mt-1 space-y-1 border-l border-stroke-divider pl-3">
      <div v-if="!items.length" class="py-1">{{ $t("spaces.assistant.agents.working") }}</div>
      <div v-for="item in items" :key="item.id" class="min-w-0 py-1">
        <p v-if="item.status === 'commentary'" class="whitespace-pre-wrap text-fg-primary">{{ item.title }}</p>
        <component :is="hasDetails(item) ? 'details' : 'div'" v-else class="group/tool min-w-0" @toggle="toggleTool($event, item.id)">
          <component :is="hasDetails(item) ? 'summary' : 'div'" class="flex list-none items-center gap-2 rounded-md py-1" :class="hasDetails(item) ? 'cursor-pointer select-none hover:text-fg-primary focus-visible:outline-2 focus-visible:outline-stroke-focus' : ''">
            <LoaderCircle v-if="active && (item.status === 'in_progress' || item.status === 'pending')" class="size-3.5 shrink-0 animate-spin" />
            <Circle v-else-if="item.status === 'failed' || item.status === 'in_progress' || item.status === 'pending'" class="size-3.5 shrink-0 text-fg-danger" />
            <Shield v-else-if="item.status === 'permission'" class="size-3.5 shrink-0" />
            <Wrench v-else class="size-3.5 shrink-0" />
            <span class="min-w-0 flex-1 truncate">{{ item.title }}</span>
            <ChevronRight v-if="hasDetails(item)" class="size-3.5 shrink-0 transition-transform group-open/tool:rotate-90" />
          </component>
          <div v-if="expanded.has(item.id) && hasDetails(item)" class="min-w-0 pl-5 pb-1 text-fg-tertiary">
            <p v-if="item.detail !== undefined" class="wrap-break-word whitespace-pre-wrap">{{ resultText(item.detail) }}</p>
            <SpaceConversationSearchMatches v-if="item.resultName === 'search-knowledge'" :value="item.result" />
            <pre v-else-if="item.result !== undefined" class="wrap-break-word whitespace-pre-wrap font-mono text-caption-sm">{{ resultText(item.result) }}</pre>
            <button
              v-if="createdNodeId(item.result)"
              type="button"
              class="mt-1 text-fg-accent hover:underline focus-visible:outline-2 focus-visible:outline-stroke-focus"
              @click="store.requestReveal([createdNodeId(item.result)!], 'focus')"
            >{{ $t("spaces.assistant.actions.viewCreated") }}</button>
          </div>
        </component>
      </div>
    </div>
  </details>
</template>
