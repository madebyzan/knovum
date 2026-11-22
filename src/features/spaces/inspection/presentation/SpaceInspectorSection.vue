<script setup lang="ts">
import { ChevronDown } from "@lucide/vue";
import { CollapsibleContent, CollapsibleRoot, CollapsibleTrigger } from "reka-ui";
import { shallowRef, type Component } from "vue";

const props = withDefaults(
  defineProps<{
    title: string;
    icon?: Component;
    count?: number | undefined;
    defaultExpanded?: boolean;
    collapsible?: boolean;
  }>(),
  {
    defaultExpanded: true,
    collapsible: true,
  },
);
const expanded = shallowRef(props.defaultExpanded);
</script>

<template>
  <CollapsibleRoot v-if="collapsible" v-model:open="expanded" as-child>
    <section class="mx-4 border-t border-stroke-divider/70 py-2">
      <div class="-mx-1.5 flex min-h-interaction items-center gap-1">
        <CollapsibleTrigger
          class="flex min-h-interaction min-w-0 flex-1 items-center gap-2 rounded-md px-1.5 text-left transition-colors duration-100 hover:bg-surface-control-hover focus-visible:outline-2 focus-visible:outline-stroke-focus"
        >
          <component :is="icon" v-if="icon" class="size-3.5 shrink-0 text-fg-tertiary" :stroke-width="1.6" />
          <span class="flex min-w-0 flex-1 items-baseline gap-1.5">
            <span class="truncate text-caption font-semibold text-fg-secondary">{{ title }}</span>
            <span
              v-if="count !== undefined && count > 0"
              class="shrink-0 text-caption-sm font-normal text-fg-tertiary tabular-nums"
            >
              {{ count }}
            </span>
          </span>
          <ChevronDown
            class="size-3 shrink-0 text-fg-tertiary transition-transform duration-150 ease-standard"
            :class="expanded && 'rotate-180'"
            :stroke-width="1.6"
          />
        </CollapsibleTrigger>
        <div v-if="$slots.actions" class="flex shrink-0 items-center gap-0.5 pr-1">
          <slot name="actions" />
        </div>
      </div>
      <CollapsibleContent class="pt-1.5 pb-1.5">
        <slot />
      </CollapsibleContent>
    </section>
  </CollapsibleRoot>
  <section v-else class="mx-4 border-t border-stroke-divider/70 py-2">
    <header class="flex min-h-interaction items-center gap-2">
      <component :is="icon" v-if="icon" class="size-3.5 shrink-0 text-fg-tertiary" :stroke-width="1.6" />
      <h3 class="flex min-w-0 flex-1 items-baseline gap-1.5">
        <span class="truncate text-caption font-semibold text-fg-secondary">{{ title }}</span>
        <span
          v-if="count !== undefined && count > 0"
          class="shrink-0 text-caption-sm font-normal text-fg-tertiary tabular-nums"
        >
          {{ count }}
        </span>
      </h3>
      <div v-if="$slots.actions" class="flex shrink-0 items-center gap-0.5">
        <slot name="actions" />
      </div>
    </header>
    <div class="pt-1.5 pb-1.5">
      <slot />
    </div>
  </section>
</template>
