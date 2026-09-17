<script setup lang="ts">
import { take } from "es-toolkit";
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";

import type { SpaceTiptapHeading } from "../../editor/tiptap-outline";

const props = defineProps<{ headings: readonly SpaceTiptapHeading[]; activeId?: string | undefined }>();
const emit = defineEmits<{ select: [position: number] }>();
const { t } = useI18n();
const open = ref(false);

// The minibar is for high-level spatial orientation — show only H1 and H2 up to 16 items
// so long documents never render a tall, cluttered "comb" or scrollbar on the right.
const minibarHeadings = computed(() => {
  const major = props.headings.filter((h) => h.level <= 2);
  const pool = major.length >= 3 ? major : props.headings;
  return take(pool, 16);
});
</script>

<template>
  <div
    v-if="headings.length"
    class="pointer-events-auto absolute top-5 right-3 z-10 hidden xl:block"
    @mouseenter="open = true"
    @mouseleave="open = false"
  >
    <div
      v-if="!open"
      class="flex w-8 flex-col items-end gap-1 rounded-control px-1.5 py-2.5 text-foreground-tertiary transition-colors hover:bg-control-hover hover:text-foreground-secondary"
      :aria-label="t('spaces.navigation.editorOutline')"
      role="button"
      tabindex="0"
      @focus="open = true"
    >
      <span
        v-for="heading in minibarHeadings"
        :key="heading.id"
        class="h-0.5 rounded-full bg-current transition-[width,color] duration-100"
        :class="heading.id === props.activeId ? 'text-foreground' : ''"
        :style="{ width: `${Math.max(8, 22 - (heading.level - 1) * 3)}px` }"
      />
    </div>
    <aside
      v-else
      class="w-72 rounded-dialog border border-border bg-overlay/96 p-2.5 shadow-popover backdrop-blur"
    >
      <nav
        class="max-h-112 scrollbar-none space-y-0.5 overflow-y-auto [&::-webkit-scrollbar]:hidden"
        :aria-label="t('spaces.navigation.editorOutline')"
      >
        <button
          v-for="heading in headings"
          :key="heading.id"
          type="button"
          class="group flex h-(--knovum-compact-item-height) w-full items-center gap-2 truncate rounded-control pr-2 text-left text-caption text-foreground-secondary outline-none hover:bg-control-hover hover:text-foreground data-[active=true]:bg-(--knovum-sidebar-item-background-selected) data-[active=true]:font-medium data-[active=true]:text-foreground"
          :style="{ paddingLeft: `${8 + (heading.level - 1) * 12}px` }"
          :data-active="heading.id === props.activeId"
          @click="emit('select', heading.position)"
        >
          <span class="truncate">{{ heading.text }}</span>
        </button>
      </nav>
    </aside>
  </div>
</template>
