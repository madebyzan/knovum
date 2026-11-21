<script setup lang="ts">
import { PopoverContent, PopoverRoot, PopoverTrigger } from "reka-ui";
import { computed, nextTick, shallowRef, useTemplateRef } from "vue";

export interface UiOutlineItem {
  id: string;
  label: string;
  level: number;
}

const props = withDefaults(
  defineProps<{
    items: readonly UiOutlineItem[];
    activeId?: string | undefined;
    label: string;
    alwaysVisible?: boolean;
    testId?: string | undefined;
  }>(),
  { alwaysVisible: true },
);

const emit = defineEmits<{ select: [id: string] }>();
const open = shallowRef(false);
const trigger = useTemplateRef<HTMLButtonElement>("trigger");

defineExpose({
  focus: async () => {
    open.value = props.items.length > 0;
    await nextTick();
    trigger.value?.focus();
  },
});

// Keep long documents' orientation marks quiet; the expanded directory includes every item.
const minibarItems = computed(() => {
  const major = props.items.filter((item) => item.level <= 2);
  return (major.length >= 3 ? major : props.items).slice(0, 16);
});

const minLevel = computed(() => {
  let min = Infinity;
  for (const item of props.items) {
    if (item.level < min) min = item.level;
  }
  return Number.isFinite(min) ? min : 1;
});

const relativeDepth = (level: number) => Math.max(0, level - minLevel.value);
</script>

<template>
  <div
    v-if="items.length"
    data-reader-ignore-selection
    class="pointer-events-auto absolute top-4 right-3.5 z-10 select-none"
    :class="alwaysVisible || open ? 'block' : 'hidden xl:block'"
    :data-testid="testId"
    @mouseenter="open = true"
    @mouseleave="open = false"
  >
    <PopoverRoot v-model:open="open" :modal="false">
      <PopoverTrigger as-child>
        <button
          ref="trigger"
          type="button"
          class="flex w-7 flex-col items-end gap-1.25 rounded-md px-1.5 py-2 transition-colors hover:bg-surface-control-hover"
          :aria-label="label"
        >
          <span
            v-for="item in minibarItems"
            :key="item.id"
            class="h-0.5 rounded-full transition-[width,background-color] duration-120"
            :class="item.id === activeId ? 'bg-fg-primary' : 'bg-fg-tertiary/45'"
            :style="{ width: `${Math.max(8, 18 - relativeDepth(item.level) * 3)}px` }"
          />
        </button>
      </PopoverTrigger>
      <PopoverContent
        side="left"
        align="start"
        :side-offset="-28"
        class="z-30 w-68 rounded-xl border border-stroke-default bg-surface-overlay/96 p-2 shadow-popover backdrop-blur"
        @open-auto-focus.prevent
        @close-auto-focus.prevent
      >
        <nav
          class="max-h-112 [scrollbar-width:none] space-y-0.5 overflow-y-auto overscroll-contain [&::-webkit-scrollbar]:hidden"
          :aria-label="label"
        >
          <button
            v-for="item in items"
            :key="item.id"
            type="button"
            class="flex h-interaction w-full items-center gap-2 rounded-md pr-2.5 text-left text-caption text-fg-secondary transition-colors hover:bg-surface-control-hover hover:text-fg-primary data-[active=true]:bg-(--knovum-sidebar-item-bg-selected) data-[active=true]:font-medium data-[active=true]:text-fg-primary"
            :style="{ paddingLeft: `${8 + relativeDepth(item.level) * 12}px` }"
            :data-active="item.id === activeId"
            :aria-current="item.id === activeId ? 'location' : undefined"
            @click="emit('select', item.id)"
          >
            <span class="min-w-0 flex-1 truncate">{{ item.label }}</span>
          </button>
        </nav>
      </PopoverContent>
    </PopoverRoot>
  </div>
</template>
