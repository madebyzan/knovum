<script setup lang="ts">
import { useVirtualizer } from "@tanstack/vue-virtual";
import { computed, ref, type ComponentPublicInstance } from "vue";

import UiButton from "../../../../shared/ui/UiButton.vue";
import type { AssistantMessage } from "../../assistant/assistant-client";

const props = defineProps<{
  messages: readonly AssistantMessage[];
  busy: boolean;
  error?: string | undefined;
  streamedText: string;
  hasOlder: boolean;
  loadingOlder: boolean;
}>();
const emit = defineEmits<{
  loadOlder: [];
  branch: [message: AssistantMessage];
  regenerate: [message: AssistantMessage];
}>();
const scroller = ref<HTMLDivElement | null>(null);
const virtualizer = useVirtualizer(
  computed(() => ({
    count: props.messages.length,
    getScrollElement: () => scroller.value,
    estimateSize: () => 92,
    overscan: 6,
  })),
);
const rows = computed(() => virtualizer.value.getVirtualItems());
const totalSize = computed(() => virtualizer.value.getTotalSize());

function text(message: AssistantMessage): string {
  return message.content.parts
    .map((part) => {
      if (part.type === "text") return part.text;
      if (part.type === "tool_result") return `${part.name}\n${JSON.stringify(part.result, null, 2)}`;
      if (part.type === "tool_call") return `${part.name}\n${JSON.stringify(part.arguments, null, 2)}`;
      return part.name ?? part.mediaType;
    })
    .join("\n");
}

function measureElement(element: Element | ComponentPublicInstance | null) {
  const row = element instanceof Element ? element : element?.$el;
  if (row instanceof Element) virtualizer.value.measureElement(row);
}
</script>

<template>
  <div class="flex h-full min-h-0 flex-col" aria-live="polite">
    <div v-if="hasOlder" class="shrink-0 p-2 text-center">
      <UiButton size="sm" variant="ghost" :disabled="loadingOlder" @click="emit('loadOlder')">
        {{ $t("spaces.assistant.loadOlder") }}
      </UiButton>
    </div>
    <div v-if="messages.length === 0 && !busy" class="text-meta py-10 text-center text-foreground-tertiary">
      {{ $t("spaces.assistant.builtInEmpty") }}
    </div>
    <div ref="scroller" class="min-h-0 flex-1 overflow-y-auto px-3">
      <div class="relative w-full" :style="{ height: `${totalSize}px` }">
        <article
          v-for="row in rows"
          :key="messages[row.index]!.id"
          :ref="measureElement"
          :data-index="row.index"
          class="absolute top-0 left-0 w-full rounded-panel border border-border/70 px-3 py-2.5"
          :class="messages[row.index]!.role === 'user' ? 'bg-control' : 'bg-raised'"
          :style="{ transform: `translateY(${row.start}px)` }"
        >
          <p class="mb-1 text-micro font-medium tracking-wide text-foreground-tertiary uppercase">
            {{ $t(`spaces.assistant.role.${messages[row.index]!.role}`) }}
          </p>
          <p class="text-label leading-relaxed whitespace-pre-wrap text-foreground">
            {{ text(messages[row.index]!) }}
          </p>
          <UiButton
            v-if="messages[row.index]!.role === 'assistant'"
            class="mt-1"
            size="sm"
            variant="ghost"
            @click="emit('branch', messages[row.index]!)"
          >
            {{ $t("spaces.assistant.branchConversation") }}
          </UiButton>
          <UiButton
            v-if="messages[row.index]!.role === 'assistant'"
            class="mt-1"
            size="sm"
            variant="ghost"
            @click="emit('regenerate', messages[row.index]!)"
          >
            {{ $t("spaces.assistant.regenerate") }}
          </UiButton>
        </article>
      </div>
      <article v-if="streamedText" class="rounded-panel border border-border/70 bg-raised px-3 py-2.5">
        <p class="mb-1 text-micro font-medium tracking-wide text-foreground-tertiary uppercase">
          {{ $t("spaces.assistant.role.assistant") }}
        </p>
        <p class="text-label leading-relaxed whitespace-pre-wrap text-foreground">{{ streamedText }}</p>
      </article>
      <p v-if="busy" class="text-meta py-2 text-foreground-tertiary">
        {{ $t("spaces.assistant.agentWorking") }}
      </p>
      <p v-if="error" class="text-meta rounded-control bg-danger-subtle p-2 text-danger">{{ error }}</p>
    </div>
  </div>
</template>
