<script setup lang="ts">
import { ChevronDown } from "@lucide/vue";
import { useToggle } from "@vueuse/core";
import { ref, watch } from "vue";

const props = withDefaults(
  defineProps<{ title: string; icon?: unknown; defaultExpanded?: boolean; padded?: boolean }>(),
  { defaultExpanded: true, padded: true },
);
const expanded = ref(props.defaultExpanded);
const toggleExpanded = useToggle(expanded);

watch(
  () => props.defaultExpanded,
  (value) => {
    expanded.value = value;
  },
);
</script>

<template>
  <div class="border-t border-divider first:border-t-0">
    <button
      type="button"
      class="flex h-(--knovum-compact-item-height) w-full items-center gap-2 px-3 text-left hover:bg-control-hover"
      :aria-expanded="expanded"
      @click="toggleExpanded()"
    >
      <component
        :is="icon"
        v-if="icon"
        class="size-(--knovum-icon-size-xs) shrink-0 text-foreground-tertiary"
        :stroke-width="1.6"
      />
      <span class="min-w-0 flex-1 text-caption font-semibold text-foreground-secondary">{{ title }}</span>
      <ChevronDown
        class="size-(--knovum-icon-size-xs) shrink-0 text-foreground-tertiary transition-transform duration-150 ease-standard"
        :class="expanded ? 'rotate-180' : ''"
        :stroke-width="1.6"
      />
    </button>
    <div
      v-show="expanded"
      class="transition-all duration-150 ease-standard"
      :class="padded ? 'px-3 pt-0.5 pb-3' : 'pb-3'"
    >
      <slot />
    </div>
  </div>
</template>
