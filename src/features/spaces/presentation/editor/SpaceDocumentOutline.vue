<script setup lang="ts">
import { PopoverContent, PopoverRoot, PopoverTrigger } from "reka-ui";
import { computed, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import type { SpaceTiptapHeading } from "../../editor/tiptap-outline";

const props = defineProps<{ headings: readonly SpaceTiptapHeading[]; activeId?: string | undefined }>();
const emit = defineEmits<{ select: [position: number] }>();
const { t } = useI18n();
const open = shallowRef(false);

// Keep long documents' orientation marks quiet; the expanded directory includes every heading.
const minibarHeadings = computed(() => {
  const major = props.headings.filter((heading) => heading.level <= 2);
  return (major.length >= 3 ? major : props.headings).slice(0, 16);
});
</script>

<template>
  <div
    v-if="headings.length"
    class="pointer-events-auto absolute top-5 right-4 z-10 hidden xl:block"
    data-testid="document-outline"
    @mouseenter="open = true"
    @mouseleave="open = false"
  >
    <PopoverRoot v-model:open="open" :modal="false">
      <PopoverTrigger as-child>
        <button
          type="button"
          class="flex w-8 flex-col items-end gap-1 rounded-md px-1.5 py-2.5 text-fg-tertiary transition-colors hover:bg-surface-control-hover hover:text-fg-secondary"
          :aria-label="t('spaces.sidebar.editorOutline.title')"
        >
          <span
            v-for="heading in minibarHeadings"
            :key="heading.id"
            class="h-0.5 rounded-full bg-current transition-[width,color] duration-100"
            :class="heading.id === activeId ? 'text-fg-primary' : ''"
            :style="{ width: `${Math.max(8, 22 - (heading.level - 1) * 3)}px` }"
          />
        </button>
      </PopoverTrigger>
      <PopoverContent
        side="left"
        align="start"
        :side-offset="-32"
        class="w-72 rounded-xl border border-stroke-default bg-surface-overlay/96 p-2.5 shadow-popover backdrop-blur"
        @open-auto-focus.prevent
        @close-auto-focus.prevent
      >
        <nav
          class="max-h-112 [scrollbar-width:none] space-y-0.5 overflow-y-auto overscroll-contain [&::-webkit-scrollbar]:hidden"
          :aria-label="t('spaces.sidebar.editorOutline.title')"
        >
          <button
            v-for="heading in headings"
            :key="heading.id"
            type="button"
            class="flex h-interaction w-full items-center rounded-md pr-2 text-left text-caption text-fg-secondary hover:bg-surface-control-hover hover:text-fg-primary data-[active=true]:bg-(--knovum-sidebar-item-bg-selected) data-[active=true]:text-fg-primary"
            :style="{ paddingLeft: `${8 + (heading.level - 1) * 12}px` }"
            :data-active="heading.id === activeId"
            :aria-current="heading.id === activeId ? 'location' : undefined"
            @click="emit('select', heading.position)"
          >
            <span class="truncate">{{ heading.text }}</span>
          </button>
        </nav>
      </PopoverContent>
    </PopoverRoot>
  </div>
</template>
