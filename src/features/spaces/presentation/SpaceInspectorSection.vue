<script setup lang="ts">
import { ChevronDown } from "@lucide/vue";
import { useToggle } from "@vueuse/core";
import { shallowRef, watch } from "vue";

const props = withDefaults(
  defineProps<{ title: string; icon?: unknown; defaultExpanded?: boolean; padded?: boolean }>(),
  { defaultExpanded: true, padded: true },
);
const expanded = shallowRef(props.defaultExpanded);
const mounted = shallowRef(props.defaultExpanded);
const toggleExpanded = useToggle(expanded);

watch(expanded, (value) => {
  if (value) mounted.value = true;
});

watch(
  () => props.defaultExpanded,
  (value) => {
    expanded.value = value;
  },
);
</script>

<template>
  <section class="mx-4 border-t border-divider">
    <button
      type="button"
      class="flex min-h-(--knovum-compact-item-height) w-full items-center gap-2 py-2 text-left hover:text-foreground"
      :aria-expanded="expanded"
      @click="toggleExpanded()"
    >
      <component
        :is="icon"
        v-if="icon"
        class="size-(--knovum-icon-size-xs) shrink-0 text-foreground-tertiary"
        :stroke-width="1.6"
      />
      <span class="min-w-0 flex-1 text-caption font-semibold text-foreground">{{ title }}</span>
      <ChevronDown
        class="size-(--knovum-icon-size-xs) shrink-0 text-foreground-tertiary transition-transform duration-150 ease-standard"
        :class="expanded ? 'rotate-180' : ''"
        :stroke-width="1.6"
      />
    </button>
    <div
      v-if="mounted"
      v-show="expanded"
      class="transition-all duration-150 ease-standard"
      :class="padded ? 'pt-1 pb-4' : 'pb-4'"
    >
      <slot />
    </div>
  </section>
</template>
