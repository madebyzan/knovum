<script setup lang="ts">
import { Check, Circle } from "@lucide/vue";
import { computed } from "vue";
import { z } from "zod";

import type { AgentPlanItem } from "../../contracts/generated/AgentPlanItem";
import type { RunEvent } from "../../contracts/generated/RunEvent";
import SpaceConversationShimmerText from "./SpaceConversationShimmerText.vue";

const props = defineProps<{ events: readonly RunEvent[]; active?: boolean }>();
const planPayload = z.strictObject({
  items: z.array(
    z.strictObject({ content: z.string(), state: z.enum(["pending", "in_progress", "completed"]) }),
  ),
});
const plan = computed<AgentPlanItem[]>(() => {
  const event = props.events.findLast(({ kind }) => kind === "builtin_agent_plan_updated");
  if (!event) return [];
  return planPayload.parse(event.payload).items;
});
</script>

<template>
  <section
    v-if="plan.length"
    class="space-y-1 border-l border-stroke-divider pl-3 text-caption text-fg-secondary"
    :aria-label="$t('spaces.conversation.activity.plan')"
  >
    <div v-for="(item, index) in plan" :key="index" class="flex items-start gap-2 py-1">
      <SpaceConversationShimmerText v-if="active && item.state === 'in_progress'" indicator>
        {{ item.content }}
      </SpaceConversationShimmerText>
      <template v-else>
        <Check v-if="item.state === 'completed'" class="mt-0.5 size-3.5 shrink-0 text-fg-tertiary" />
        <Circle v-else class="mt-0.5 size-3.5 shrink-0 text-fg-tertiary" />
        <span :class="item.state === 'completed' ? 'text-fg-tertiary' : ''">{{ item.content }}</span>
      </template>
    </div>
  </section>
</template>
