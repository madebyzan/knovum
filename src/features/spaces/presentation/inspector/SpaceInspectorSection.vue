<script setup lang="ts">
import { ChevronDown } from "@lucide/vue";
import { CollapsibleContent, CollapsibleRoot, CollapsibleTrigger } from "reka-ui";
import { shallowRef, watch } from "vue";

const props = withDefaults(
  defineProps<{ title: string; icon?: unknown; defaultExpanded?: boolean; padded?: boolean }>(),
  { defaultExpanded: true, padded: true },
);
const expanded = shallowRef(props.defaultExpanded);

watch(
  () => props.defaultExpanded,
  (value) => {
    expanded.value = value;
  },
);
</script>

<template>
  <CollapsibleRoot v-model:open="expanded" as-child>
    <section class="mx-4 border-t border-stroke-divider">
      <CollapsibleTrigger class="flex w-full items-center gap-2 py-2 text-left hover:text-fg-primary">
        <component :is="icon" v-if="icon" class="size-3 shrink-0 text-fg-tertiary" :stroke-width="1.6" />
        <span class="min-w-0 flex-1 text-caption font-semibold text-fg-primary">{{ title }}</span>
        <ChevronDown
          class="size-3 shrink-0 text-fg-tertiary transition-transform duration-150 ease-standard"
          :class="expanded ? 'rotate-180' : ''"
          :stroke-width="1.6"
        />
      </CollapsibleTrigger>
      <CollapsibleContent :class="padded ? 'pt-1 pb-4' : 'pb-4'">
        <slot />
      </CollapsibleContent>
    </section>
  </CollapsibleRoot>
</template>
