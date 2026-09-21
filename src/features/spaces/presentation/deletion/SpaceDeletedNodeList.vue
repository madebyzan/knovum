<script setup lang="ts">
import { RotateCcw, Trash2 } from "@lucide/vue";
import { useVirtualizer } from "@tanstack/vue-virtual";
import { computed, useTemplateRef, type ComponentPublicInstance } from "vue";
import { useI18n } from "vue-i18n";

import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import type { SpaceDeletedNode } from "../../deletion/deleted-node";
import { nodeIcon } from "../node-icon";

const props = defineProps<{ items: readonly SpaceDeletedNode[]; busy: boolean }>();
const emit = defineEmits<{ restore: [item: SpaceDeletedNode]; remove: [item: SpaceDeletedNode] }>();
const { locale, t } = useI18n();
const scroller = useTemplateRef<HTMLDivElement>("scroller");
const dates = computed(
  () => new Intl.DateTimeFormat(locale.value, { dateStyle: "short", timeStyle: "short" }),
);
const virtualizer = useVirtualizer(
  computed(() => ({
    count: props.items.length,
    getScrollElement: () => scroller.value,
    getItemKey: (index: number) => props.items[index]!.id,
    estimateSize: () => 48,
    overscan: 6,
  })),
);
const rows = computed(() =>
  virtualizer.value.getVirtualItems().map((row) => ({ row, item: props.items[row.index]! })),
);
const totalSize = computed(() => virtualizer.value.getTotalSize());

function measureElement(element: Element | ComponentPublicInstance | null) {
  const row = element instanceof Element ? element : element?.$el;
  if (row instanceof Element) virtualizer.value.measureElement(row);
}
</script>

<template>
  <div ref="scroller" class="h-72 overflow-y-auto" :aria-busy="busy">
    <div class="relative w-full" :style="{ height: `${totalSize}px` }">
      <div
        v-for="{ row, item } in rows"
        :key="item.id"
        :ref="measureElement"
        :data-index="row.index"
        class="absolute top-0 left-0 flex min-h-12 w-full items-center gap-2 rounded-control px-2 py-1 hover:bg-control-hover"
        :style="{ transform: `translateY(${row.start}px)` }"
      >
        <component
          :is="nodeIcon(item.kind)"
          class="size-(--knovum-icon-size-sm) shrink-0 text-foreground-secondary"
        />
        <div class="min-w-0 flex-1">
          <p class="truncate text-label font-medium">{{ item.title }}</p>
          <p class="truncate text-micro text-foreground-tertiary">
            {{ item.spaceTitle }} · {{ dates.format(item.deletedAt) }}
          </p>
        </div>
        <UiIconButton
          :disabled="busy"
          :label="t('spaces.navigation.restoreItem')"
          size="sm"
          @click="emit('restore', item)"
        >
          <RotateCcw class="size-(--knovum-icon-size-xs)" />
        </UiIconButton>
        <UiIconButton
          :disabled="busy"
          :label="t('spaces.navigation.deletePermanently')"
          size="sm"
          @click="emit('remove', item)"
        >
          <Trash2 class="size-(--knovum-icon-size-xs) text-danger" />
        </UiIconButton>
      </div>
    </div>
  </div>
</template>
