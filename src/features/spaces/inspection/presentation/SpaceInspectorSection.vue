<script setup lang="ts">
import { ChevronDown } from "@lucide/vue";
import { CollapsibleContent, CollapsibleRoot, CollapsibleTrigger } from "reka-ui";
import { shallowRef, type Component } from "vue";

const props = withDefaults(defineProps<{ title: string; icon?: Component; defaultExpanded?: boolean }>(), {
  defaultExpanded: true,
});
const expanded = shallowRef(props.defaultExpanded);
</script>

<template>
  <CollapsibleRoot v-model:open="expanded" as-child>
    <section class="mx-4 border-t border-stroke-divider py-1.5">
      <CollapsibleTrigger
        class="flex min-h-interaction w-full items-center gap-2 rounded-md px-1 text-left transition-colors hover:bg-surface-control-hover focus-visible:outline-2 focus-visible:outline-stroke-focus"
      >
        <component :is="icon" v-if="icon" class="size-3.5 shrink-0 text-fg-tertiary" :stroke-width="1.6" />
        <span class="min-w-0 flex-1 text-label font-medium text-fg-primary">{{ title }}</span>
        <ChevronDown
          class="size-3 shrink-0 text-fg-tertiary transition-transform duration-150 ease-standard"
          :class="expanded ? 'rotate-180' : ''"
          :stroke-width="1.6"
        />
      </CollapsibleTrigger>
      <CollapsibleContent class="px-1 pt-1 pb-2">
        <slot />
      </CollapsibleContent>
    </section>
  </CollapsibleRoot>
</template>
