<script setup lang="ts">
import { useVirtualizer } from "@tanstack/vue-virtual";
import { computed, ref, type ComponentPublicInstance } from "vue";

import type { ExternalConversationMessage } from "../../agent-creation/external-conversation-client";

const props = defineProps<{ messages: readonly ExternalConversationMessage[] }>();
const scroller = ref<HTMLDivElement | null>(null);
const virtualizer = useVirtualizer(
  computed(() => ({
    count: props.messages.length,
    getScrollElement: () => scroller.value,
    estimateSize: () => 72,
    overscan: 8,
  })),
);
const rows = computed(() => virtualizer.value.getVirtualItems());
const totalSize = computed(() => virtualizer.value.getTotalSize());

function measure(element: Element | ComponentPublicInstance | null) {
  const row = element instanceof Element ? element : element?.$el;
  if (row instanceof Element) virtualizer.value.measureElement(row);
}
</script>

<template>
  <div ref="scroller" class="min-h-0 flex-1 overflow-y-auto px-3" aria-live="polite">
    <div v-if="messages.length" class="relative w-full" :style="{ height: `${totalSize}px` }">
      <article
        v-for="row in rows"
        :key="messages[row.index]!.id"
        :ref="measure"
        :data-index="row.index"
        class="absolute top-0 left-0 w-full rounded-panel border border-border/70 px-3 py-2.5"
        :class="messages[row.index]!.role === 'user' ? 'bg-control' : 'bg-raised'"
        :style="{ transform: `translateY(${row.start}px)` }"
      >
        <p class="mb-1 text-micro font-medium tracking-wide text-foreground-tertiary uppercase">
          {{ $t(`spaces.assistant.role.${messages[row.index]!.role}`) }}
        </p>
        <p class="text-label leading-relaxed whitespace-pre-wrap">{{ messages[row.index]!.content }}</p>
      </article>
    </div>
    <p v-else class="text-meta py-10 text-center text-foreground-tertiary">
      {{ $t("spaces.assistant.externalConversationEmpty") }}
    </p>
  </div>
</template>
