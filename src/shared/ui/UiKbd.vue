<script setup lang="ts">
import { computed, type HTMLAttributes } from "vue";

import { cn } from "./cn";
import { useKbd } from "./use-kbd";

const props = withDefaults(
  defineProps<{
    shortcut?: string | undefined;
    variant?: "subtle" | "badge" | undefined;
    class?: HTMLAttributes["class"];
  }>(),
  {
    shortcut: undefined,
    variant: "subtle",
  },
);

const { parseShortcut, formatShortcutLabel } = useKbd();

const keyGroups = computed(() => (props.shortcut ? parseShortcut(props.shortcut) : []));
const shortcutLabel = computed(() => (props.shortcut ? formatShortcutLabel(props.shortcut) : undefined));
</script>

<template>
  <span
    v-if="variant === 'badge' && keyGroups.length"
    :class="cn('inline-flex items-center gap-1 select-none', props.class)"
  >
    <template v-for="(group, gIdx) in keyGroups" :key="gIdx">
      <span v-if="group.connector" class="px-0.5 font-sans text-caption-sm text-fg-tertiary">
        {{ group.connector }}
      </span>
      <!-- h-5.5 min-w-5.5 (22px) is compact keycap badge geometry to tightly wrap micro typography -->
      <kbd
        v-for="(key, kIdx) in group.keys"
        :key="kIdx"
        class="inline-flex h-5.5 min-w-5.5 items-center justify-center rounded border border-stroke-default/80 bg-surface-muted px-1.5 font-sans text-caption-sm font-medium text-fg-primary"
      >
        {{ key }}
      </kbd>
    </template>
  </span>
  <kbd
    v-else
    :class="
      cn(
        'inline-flex min-w-5 items-center justify-center rounded-sm px-1 py-0.5 font-sans text-caption-sm leading-none text-fg-tertiary select-none',
        props.class,
      )
    "
  >
    {{ shortcutLabel }}<slot v-if="!shortcut" />
  </kbd>
</template>
