<script setup lang="ts">
import { Search } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import UiFindBar from "../../../../shared/ui/UiFindBar.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import SpaceReaderFontControls from "./SpaceReaderFontControls.vue";
import type { useDomBlockFind } from "./use-dom-block-find";
import type { useReaderTypography } from "./use-reader-typography";

withDefaults(
  defineProps<{
    find: ReturnType<typeof useDomBlockFind>;
    typography: ReturnType<typeof useReaderTypography>;
    showControls?: boolean;
  }>(),
  { showControls: true },
);

const { t } = useI18n();
</script>

<template>
  <div
    data-reader-ignore-selection
    class="flex h-8.5 shrink-0 items-center justify-between gap-2 border-b border-stroke-divider/60 bg-surface-raised px-3 select-none"
  >
    <div class="flex min-w-0 items-center gap-2">
      <slot name="leading" />
    </div>
    <div v-if="showControls" class="flex shrink-0 items-center gap-1">
      <UiIconButton
        :label="t('spaces.inspector.reader.searchPlaceholder')"
        size="sm"
        tooltip
        :class="find.open.value && 'bg-surface-control text-fg-primary'"
        @click="find.toggle()"
      >
        <Search class="size-3.5" :stroke-width="1.6" />
      </UiIconButton>
      <SpaceReaderFontControls
        :can-decrease="typography.canDecrease.value"
        :can-increase="typography.canIncrease.value"
        @decrease="typography.decrease()"
        @increase="typography.increase()"
      />
    </div>
  </div>
  <UiFindBar
    v-if="showControls && find.open.value"
    v-model="find.query.value"
    :match-current="find.matched.value.length ? find.matchIndex.value + 1 : 0"
    :match-total="find.matched.value.length"
    @next="find.next()"
    @prev="find.prev()"
    @close="find.reset()"
  />
</template>
