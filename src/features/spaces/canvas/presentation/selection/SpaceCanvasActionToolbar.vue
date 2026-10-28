<script setup lang="ts">
import { ToolbarRoot } from "reka-ui";
import { computed, toRef, useTemplateRef } from "vue";
import { useI18n } from "vue-i18n";

import { CANVAS_ACTION_BAR_CLASSES } from "../canvas-action-classes";
import { useNodeActionBarDensity } from "./use-node-action-bar-density";

/**
 * The floating bar a selection's actions sit in. It measures its content and hands the slot the
 * density that fits `availableWidth`; `identity` names that content, so changing it re-measures.
 */
const props = defineProps<{ label: string; availableWidth: number; identity: string }>();
defineSlots<{ default(props: { compact: boolean; minimal: boolean }): unknown }>();
const { locale } = useI18n();
const bar = useTemplateRef<HTMLElement>("bar");
const identity = computed(() => `${locale.value}:${props.identity}`);
const { compact, minimal, ready } = useNodeActionBarDensity(bar, toRef(props, "availableWidth"), identity);
</script>

<template>
  <div ref="bar" :class="ready ? 'visible' : 'invisible'" :style="{ maxWidth: `${availableWidth}px` }">
    <ToolbarRoot :class="CANVAS_ACTION_BAR_CLASSES" :loop="false" :aria-label="label">
      <slot :compact="compact" :minimal="minimal" />
    </ToolbarRoot>
  </div>
</template>
