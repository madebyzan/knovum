<script setup lang="ts">
import { Check, ChevronRight } from "@lucide/vue";
import { computed, nextTick, shallowRef, useTemplateRef, watch } from "vue";

import SpaceConversationShimmerText from "./SpaceConversationShimmerText.vue";

const props = defineProps<{
  text: string;
  /** The run is currently thinking right here (no visible text or tool call has followed yet). */
  active: boolean;
  /** Elapsed thinking duration in milliseconds, once this reasoning phase has settled. */
  durationMs?: number | undefined;
}>();

// null follows automatic lifecycle: compact live preview while active, collapsed when settled.
// Clicking explicitly pins open or closed until the user toggles again.
const userExpanded = shallowRef<boolean | null>(null);
const hasOverflow = shallowRef(false);
const previewRef = useTemplateRef<HTMLElement>("previewRef");
let pinnedToBottom = true;

const showFull = computed(() => userExpanded.value === true);
const showLivePreview = computed(() => props.active && userExpanded.value === null);
const bodyVisible = computed(() => showFull.value || showLivePreview.value);
const durationSeconds = computed(() =>
  props.durationMs !== undefined ? Math.round(props.durationMs / 1000) : 0,
);

function toggle() {
  userExpanded.value = !bodyVisible.value;
}

function handleScroll() {
  const el = previewRef.value;
  if (!el) return;
  hasOverflow.value = el.scrollTop > 2 && el.scrollHeight > el.clientHeight;
  pinnedToBottom = el.scrollHeight - el.scrollTop - el.clientHeight < 28;
}

watch(
  () => props.active,
  (isActive) => {
    if (isActive) pinnedToBottom = true;
  },
);

watch(
  () => props.text,
  async () => {
    if (!showLivePreview.value) return;
    await nextTick();
    const el = previewRef.value;
    if (!el) return;
    if (pinnedToBottom) el.scrollTop = el.scrollHeight;
    hasOverflow.value = el.scrollTop > 2 && el.scrollHeight > el.clientHeight;
  },
  { immediate: true },
);
</script>

<template>
  <div class="w-full text-caption text-fg-secondary" data-testid="conversation-thinking">
    <button
      type="button"
      class="-ml-1.5 flex w-fit max-w-full cursor-pointer items-center gap-1.5 rounded-md px-1.5 py-1 transition-colors select-none hover:bg-surface-control/70 hover:text-fg-primary focus-visible:outline-2 focus-visible:outline-stroke-focus"
      :aria-expanded="bodyVisible"
      @click="toggle"
    >
      <SpaceConversationShimmerText v-if="active" indicator>
        {{ $t("spaces.conversation.activity.thinking") }}
      </SpaceConversationShimmerText>
      <template v-else>
        <Check class="size-3.5 shrink-0 text-fg-tertiary" />
        <span class="truncate tabular-nums">{{
          durationSeconds >= 1
            ? $t("spaces.conversation.activity.thoughtFor", { seconds: durationSeconds })
            : $t("spaces.conversation.activity.thought")
        }}</span>
      </template>
      <ChevronRight
        class="size-3.5 shrink-0 text-fg-tertiary transition-transform duration-150"
        :class="bodyVisible && 'rotate-90'"
      />
    </button>
    <div
      v-if="bodyVisible"
      ref="previewRef"
      class="mt-1 ml-1.5 overflow-y-auto overscroll-contain border-l-2 border-stroke-divider/80 pl-3 text-caption leading-relaxed wrap-break-word whitespace-pre-wrap text-fg-tertiary"
      :class="[showFull ? 'max-h-72' : 'max-h-24', showLivePreview && hasOverflow && 'thinking-live-fade']"
      @scroll.passive="handleScroll"
    >
      {{ text.trim() }}
    </div>
  </div>
</template>

<style scoped>
.thinking-live-fade {
  mask-image: linear-gradient(180deg, transparent 0%, #000 22%, #000 100%);
  -webkit-mask-image: linear-gradient(180deg, transparent 0%, #000 22%, #000 100%);
}
</style>
