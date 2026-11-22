<script setup lang="ts">
import { ChevronDown } from "@lucide/vue";
import { useResizeObserver } from "@vueuse/core";
import { computed, nextTick, onMounted, shallowRef, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";

const props = withDefaults(
  defineProps<{
    maxHeight?: number;
    contentKey?: unknown;
  }>(),
  {
    maxHeight: 128,
  },
);

const { t } = useI18n();
const content = useTemplateRef<HTMLElement>("content");
const expanded = shallowRef(false);
const overflowing = shallowRef(false);
const clamped = computed(() => !expanded.value && overflowing.value);

function measure() {
  const el = content.value;
  if (!el) return;
  overflowing.value = el.scrollHeight > props.maxHeight + 4;
}

useResizeObserver(content, measure);

watch(
  () => props.contentKey,
  async () => {
    expanded.value = false;
    await nextTick();
    measure();
  },
);

onMounted(async () => {
  await nextTick();
  measure();
});

defineExpose({ measure });
</script>

<template>
  <div class="min-w-0">
    <div
      class="min-w-0"
      :class="clamped && 'clamp-masked overflow-hidden'"
      :style="clamped ? { maxHeight: `${maxHeight}px` } : undefined"
    >
      <div ref="content" class="min-w-0">
        <slot />
      </div>
    </div>
    <button
      v-if="overflowing || expanded"
      type="button"
      :aria-expanded="expanded"
      class="group/clamp mt-1.5 inline-flex cursor-pointer items-center gap-1 rounded-sm text-caption-sm font-medium text-fg-secondary transition-colors duration-120 select-none hover:text-fg-primary focus-visible:outline-2 focus-visible:outline-stroke-focus"
      @click="expanded = !expanded"
    >
      <span>{{ expanded ? t("common.collapse") : t("common.expand") }}</span>
      <ChevronDown
        class="size-3 text-fg-tertiary transition-transform duration-150 ease-standard group-hover/clamp:text-fg-secondary"
        :class="expanded && 'rotate-180'"
        :stroke-width="1.8"
      />
    </button>
  </div>
</template>

<style scoped>
/* True alpha mask fades clamped content seamlessly over any translucent or vibrant desktop surface. */
.clamp-masked {
  -webkit-mask-image: linear-gradient(to bottom, #000 calc(100% - 28px), transparent 100%);
  mask-image: linear-gradient(to bottom, #000 calc(100% - 28px), transparent 100%);
}
</style>
