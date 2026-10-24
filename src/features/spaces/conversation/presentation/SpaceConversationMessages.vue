<script setup lang="ts">
import { ArrowDown } from "@lucide/vue";
import { computed, nextTick, useTemplateRef, watch } from "vue";

import UiButton from "../../../../shared/ui/UiButton.vue";
import type { ConversationMessage } from "../../contracts/generated/ConversationMessage";
import type { RunEvent } from "../../contracts/generated/RunEvent";
import { conversationTurns } from "../conversation-turns";
import { useConversationScroll } from "../use-conversation-scroll";
import SpaceConversationEmptyGuide from "./SpaceConversationEmptyGuide.vue";
import SpaceConversationTurn from "./SpaceConversationTurn.vue";

const props = defineProps<{
  messages: readonly ConversationMessage[];
  eventsByRunId: Readonly<Record<string, RunEvent[]>>;
  activeRunId?: string | undefined;
  busy: boolean;
  hasDraft: boolean;
  preparing: boolean;
  producing?: boolean | undefined;
  streamedText: string;
  hasOlder: boolean;
  loadingOlder: boolean;
  scrollKey: string;
  savedScrollTop?: number | undefined;
}>();
const emit = defineEmits<{
  loadOlder: [];
  branch: [message: ConversationMessage];
  regenerate: [message: ConversationMessage];
  scrollPosition: [key: string, top: number];
  suggest: [prompt: string, selection?: string];
  inspectActivity: [runId: string];
}>();
const turns = computed(() =>
  conversationTurns({
    messages: props.messages,
    activeRunId: props.activeRunId,
    streamedText: props.streamedText,
    preparing: props.preparing,
    producing: props.producing,
  }),
);
const scroller = useTemplateRef<HTMLDivElement>("scroller");
const { atTail, rows, totalSize, measureElement, updateScrollPosition, scrollToTail, loadOlder } =
  useConversationScroll(
    scroller,
    {
      get rows() {
        return turns.value;
      },
      get streamedText() {
        return props.streamedText;
      },
      get loadingOlder() {
        return props.loadingOlder;
      },
      get scrollKey() {
        return props.scrollKey;
      },
      get savedScrollTop() {
        return props.savedScrollTop;
      },
    },
    {
      loadOlder: () => emit("loadOlder"),
      remember: (key, top) => emit("scrollPosition", key, top),
    },
  );
watch(
  () => props.eventsByRunId,
  async () => {
    if (!atTail.value) return;
    await nextTick();
    scrollToTail();
  },
);
</script>

<template>
  <div class="relative flex h-full min-h-0 flex-col">
    <div v-if="hasOlder" class="shrink-0 p-2 text-center">
      <UiButton size="sm" variant="ghost" :disabled="loadingOlder" @click="loadOlder">
        {{ $t("spaces.conversation.messages.loadOlder") }}
      </UiButton>
    </div>
    <div
      ref="scroller"
      class="min-h-0 flex-1 overflow-y-auto px-4 pt-5 pb-6"
      @scroll.passive="updateScrollPosition"
    >
      <div
        v-if="turns.length === 0 && !hasDraft && !busy"
        class="flex min-h-full items-center justify-center px-2 pb-12"
      >
        <SpaceConversationEmptyGuide @select="(value, selection) => emit('suggest', value, selection)" />
      </div>
      <div class="relative w-full" :style="{ height: `${totalSize}px` }">
        <div
          v-for="row in rows"
          :key="turns[row.index]!.id"
          :ref="measureElement"
          :data-index="row.index"
          class="absolute top-0 left-0 w-full pb-8"
          :style="{ transform: `translateY(${row.start}px)` }"
        >
          <SpaceConversationTurn
            :turn="turns[row.index]!"
            :events="turns[row.index]!.runId ? (eventsByRunId[turns[row.index]!.runId!] ?? []) : []"
            :messages="messages"
            :busy="busy"
            @branch="emit('branch', $event)"
            @regenerate="emit('regenerate', $event)"
            @inspect-activity="emit('inspectActivity', $event)"
          />
        </div>
      </div>
    </div>
    <Transition
      enter-active-class="transition-opacity duration-150"
      leave-active-class="transition-opacity duration-100"
      enter-from-class="opacity-0"
      leave-to-class="opacity-0"
    >
      <UiButton
        v-if="!atTail && turns.length > 0"
        size="sm"
        variant="secondary"
        class="shadow-overlay absolute inset-x-0 bottom-3 mx-auto w-fit gap-1 rounded-full"
        @click="scrollToTail"
      >
        <ArrowDown class="size-3.5" />
        {{ $t("spaces.conversation.messages.new") }}
      </UiButton>
    </Transition>
  </div>
</template>
