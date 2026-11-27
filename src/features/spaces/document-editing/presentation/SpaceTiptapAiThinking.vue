<script setup lang="ts">
import { Check, ChevronRight } from "@lucide/vue";
import { computed, nextTick, shallowRef, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiKnovumAiIcon from "../../../../shared/ui/UiKnovumAiIcon.vue";

const props = defineProps<{
  text: string;
  active: boolean;
  durationMs?: number | undefined;
}>();

const { t } = useI18n();
const liveBoxRef = useTemplateRef<HTMLElement>("liveBoxRef");
const expanded = shallowRef(false);
const hasOverflow = shallowRef(false);
let pinnedToBottom = true;

const trimmed = computed(() => props.text.trim());
const durationSeconds = computed(() =>
  props.durationMs !== undefined ? Math.round(props.durationMs / 1000) : 0,
);

function handleScroll() {
  const box = liveBoxRef.value;
  if (!box) return;
  hasOverflow.value = box.scrollTop > 2 && box.scrollHeight > box.clientHeight;
  pinnedToBottom = box.scrollHeight - box.scrollTop - box.clientHeight < 28;
}

watch(
  () => props.active,
  (isActive) => {
    if (!isActive) return;
    pinnedToBottom = true;
    expanded.value = false;
    hasOverflow.value = false;
  },
);

watch(
  () => props.text,
  async () => {
    if (!props.active) return;
    await nextTick();
    const box = liveBoxRef.value;
    if (!box) return;
    if (pinnedToBottom) box.scrollTop = box.scrollHeight;
    hasOverflow.value = box.scrollTop > 2 && box.scrollHeight > box.clientHeight;
  },
  { immediate: true },
);
</script>

<template>
  <div v-if="active" class="space-y-2" data-testid="tiptap-ai-thinking-live">
    <div class="flex items-center gap-2 text-caption text-fg-secondary">
      <UiKnovumAiIcon class="size-3.5 animate-pulse text-fg-secondary" />
      <span>{{ t("spaces.richtext.ai.panel.thinking") }}</span>
    </div>
    <div
      v-if="trimmed"
      ref="liveBoxRef"
      class="ml-1.5 max-h-20 overflow-y-auto overscroll-contain border-l-2 border-stroke-divider/80 pl-2.5 text-caption leading-relaxed wrap-break-word whitespace-pre-wrap text-fg-tertiary"
      :class="hasOverflow && 'thinking-live-fade'"
      @scroll.passive="handleScroll"
    >
      {{ trimmed }}
    </div>
  </div>
  <div v-else-if="trimmed" class="mb-2" data-testid="tiptap-ai-thinking">
    <button
      type="button"
      class="-ml-1.5 flex w-fit max-w-full cursor-pointer items-center gap-1.5 rounded-md px-1.5 py-0.5 text-caption text-fg-secondary transition-colors select-none hover:bg-surface-control/70 hover:text-fg-primary focus-visible:outline-2 focus-visible:outline-stroke-focus"
      :aria-expanded="expanded"
      @click="expanded = !expanded"
    >
      <Check class="size-3.5 shrink-0 text-fg-tertiary" />
      <span class="truncate tabular-nums">{{
        durationSeconds >= 1
          ? t("spaces.richtext.ai.panel.thoughtFor", { seconds: durationSeconds })
          : t("spaces.richtext.ai.panel.thought")
      }}</span>
      <ChevronRight
        class="size-3.5 shrink-0 text-fg-tertiary transition-transform duration-150"
        :class="expanded && 'rotate-90'"
      />
    </button>
    <div
      v-if="expanded"
      class="mt-1 ml-1.5 max-h-36 overflow-y-auto overscroll-contain border-l-2 border-stroke-divider/80 pl-2.5 text-caption leading-relaxed wrap-break-word whitespace-pre-wrap text-fg-tertiary"
    >
      {{ trimmed }}
    </div>
  </div>
</template>

<style scoped>
.thinking-live-fade {
  mask-image: linear-gradient(180deg, transparent 0%, #000 22%, #000 100%);
  -webkit-mask-image: linear-gradient(180deg, transparent 0%, #000 22%, #000 100%);
}
</style>
