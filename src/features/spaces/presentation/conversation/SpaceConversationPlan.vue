<script setup lang="ts">
import { Check, Circle, LoaderCircle } from "@lucide/vue";
import { computed } from "vue";
import { z } from "zod";

import type { AssistantPlanItem } from "../../contracts/generated/AssistantPlanItem";
import type { RunEvent } from "../../contracts/generated/RunEvent";

const props = defineProps<{ events: readonly RunEvent[] }>();
const planPayload = z.strictObject({
  items: z.array(
    z.strictObject({ content: z.string(), state: z.enum(["pending", "in_progress", "completed"]) }),
  ),
});
const plan = computed<AssistantPlanItem[]>(() => {
  const event = props.events.findLast(({ kind }) => kind === "assistant_plan_updated");
  if (!event) return [];
  return planPayload.parse(event.payload).items;
});
</script>

<template>
  <section
    v-if="plan.length"
    class="mt-3 space-y-1 border-l border-stroke-divider pl-3 text-caption text-fg-secondary"
    :aria-label="$t('spaces.assistant.activity.plan')"
  >
    <div v-for="(item, index) in plan" :key="index" class="flex items-start gap-2 py-1">
      <Check v-if="item.state === 'completed'" class="mt-0.5 size-3 shrink-0" />
      <LoaderCircle v-else-if="item.state === 'in_progress'" class="mt-0.5 size-3 shrink-0" />
      <Circle v-else class="mt-0.5 size-3 shrink-0" />
      <span>{{ item.content }}</span>
    </div>
  </section>
</template>
