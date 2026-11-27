<script setup lang="ts">
import type { Component } from "vue";

import type { SettingsPaneId } from "./settings-pane";

defineProps<{
  id: SettingsPaneId;
  icon: Component;
  label: string;
  active: boolean;
  badge?: string | undefined;
}>();

const emit = defineEmits<{ select: [pane: SettingsPaneId] }>();
</script>

<template>
  <button
    :data-testid="`settings-pane-${id}`"
    type="button"
    class="mb-0.5 flex h-(--knovum-control-height-md) w-full items-center gap-2.5 rounded-md px-2 text-left text-body-lg text-fg-secondary transition-colors duration-100 hover:bg-(--knovum-sidebar-item-bg-hover) hover:text-fg-primary data-[active=true]:bg-(--knovum-sidebar-item-bg-selected) data-[active=true]:font-medium data-[active=true]:text-fg-primary"
    :data-active="active"
    :aria-current="active ? 'page' : undefined"
    @click="emit('select', id)"
  >
    <component :is="icon" class="size-4 shrink-0" :stroke-width="1.6" />
    <span class="min-w-0 flex-1 truncate">{{ label }}</span>
    <span
      v-if="badge"
      data-testid="settings-about-update-badge"
      class="shrink-0 text-caption-sm font-medium text-fg-accent tabular-nums"
    >
      {{ badge }}
    </span>
  </button>
</template>
