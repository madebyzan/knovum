<script setup lang="ts">
import { ChevronDown } from "@lucide/vue";
import { CollapsibleContent, CollapsibleRoot, CollapsibleTrigger } from "reka-ui";
import { computed, inject, provide, watch } from "vue";

import { overlayScopeVisibleKey } from "../../../../shared/contracts/overlay-scope-visibility";

// Settings retain their draft and running operation when the section is collapsed.
const isExpanded = defineModel<boolean>("isExpanded", { default: false });
const parentVisibility = inject(overlayScopeVisibleKey, undefined);
provide(
  overlayScopeVisibleKey,
  computed(() => isExpanded.value && parentVisibility?.value !== false),
);
const props = defineProps<{ title: string; isInvalid?: boolean }>();
watch(
  () => props.isInvalid,
  (isInvalid) => {
    if (isInvalid) isExpanded.value = true;
  },
  { immediate: true },
);
</script>

<template>
  <CollapsibleRoot
    v-model:open="isExpanded"
    class="border-t border-stroke-divider pt-1 first:border-t-0 first:pt-0"
  >
    <CollapsibleTrigger
      class="flex h-8 w-full items-center gap-2 rounded-md px-1 text-left text-label font-medium text-fg-secondary hover:bg-surface-control-hover hover:text-fg-primary focus-visible:outline-2 focus-visible:outline-stroke-focus"
    >
      <span class="min-w-0 flex-1">{{ title }}</span>
      <ChevronDown
        class="size-3.5 transition-transform duration-150 ease-standard"
        :class="isExpanded ? 'rotate-180' : ''"
        :stroke-width="1.6"
      />
    </CollapsibleTrigger>
    <CollapsibleContent v-show="isExpanded" force-mount class="space-y-3 pt-2 pb-1">
      <slot />
    </CollapsibleContent>
  </CollapsibleRoot>
</template>
