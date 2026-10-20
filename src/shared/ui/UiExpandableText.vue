<script setup lang="ts">
import { ChevronDown } from "@lucide/vue";
import { useResizeObserver } from "@vueuse/core";
import { computed, nextTick, onMounted, shallowRef, useTemplateRef, watch, type HTMLAttributes } from "vue";
import { useI18n } from "vue-i18n";

import { cn } from "./cn";

const CLASSES_BY_LINE_COUNT: Record<number, string> = {
  1: "line-clamp-1",
  2: "line-clamp-2",
  3: "line-clamp-3",
  4: "line-clamp-4",
  5: "line-clamp-5",
  6: "line-clamp-6",
};

const props = withDefaults(
  defineProps<{
    text?: string | undefined;
    clampLines?: number | undefined;
    preformatted?: boolean | undefined;
    class?: HTMLAttributes["class"];
    expandLabel?: string | undefined;
    collapseLabel?: string | undefined;
  }>(),
  {
    clampLines: 4,
    preformatted: false,
  },
);

const { t } = useI18n();
const textRef = useTemplateRef<HTMLElement>("textRef");
const expanded = shallowRef(false);
const hasOverflow = shallowRef(false);

function checkOverflow() {
  if (expanded.value) return;
  const el = textRef.value;
  if (!el) return;
  hasOverflow.value = el.scrollHeight > el.clientHeight + 1;
}

useResizeObserver(textRef, () => {
  checkOverflow();
});

watch(
  () => [props.text, props.clampLines],
  async () => {
    expanded.value = false;
    await nextTick();
    checkOverflow();
  },
);

onMounted(async () => {
  await nextTick();
  checkOverflow();
});

const clampClass = computed(() => {
  if (expanded.value) return undefined;
  return CLASSES_BY_LINE_COUNT[props.clampLines] ?? "line-clamp-4";
});

function toggle() {
  expanded.value = !expanded.value;
  if (!expanded.value) {
    nextTick(checkOverflow);
  }
}

defineExpose({
  expanded,
  hasOverflow,
  checkOverflow,
  toggle,
});
</script>

<template>
  <div class="relative">
    <component
      :is="preformatted ? 'pre' : 'p'"
      ref="textRef"
      :class="
        cn(
          clampClass,
          preformatted ? 'font-sans whitespace-pre-wrap select-text' : 'select-text',
          props.class,
        )
      "
    >
      <slot>{{ text }}</slot>
    </component>

    <button
      v-if="hasOverflow || expanded"
      type="button"
      :aria-expanded="expanded"
      class="group/expand mt-1 inline-flex cursor-pointer items-center gap-1 rounded-xs text-caption-sm font-medium text-fg-secondary transition-colors duration-150 select-none hover:text-fg-primary focus-visible:outline-2 focus-visible:outline-stroke-focus"
      @click="toggle"
    >
      <span>{{
        expanded ? (collapseLabel ?? t("common.collapse")) : (expandLabel ?? t("common.expand"))
      }}</span>
      <ChevronDown
        class="size-3 text-fg-tertiary transition-transform duration-200 ease-standard group-hover/expand:text-fg-secondary"
        :class="expanded ? 'rotate-180' : ''"
        :stroke-width="1.8"
      />
    </button>
  </div>
</template>
