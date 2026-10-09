<script setup lang="ts">
import { ArrowLeft } from "@lucide/vue";
import type { Component } from "vue";

import UiButton from "../../../shared/ui/UiButton.vue";

defineProps<{ name: string; description?: string | null | undefined; icon: Component }>();
const emit = defineEmits<{ back: [] }>();
</script>

<template>
  <header class="space-y-4">
    <UiButton size="sm" variant="ghost" class="-ml-2.5" data-testid="extension-back" @click="emit('back')">
      <ArrowLeft class="size-3.5" />{{ $t("extensions.overview.back") }}
    </UiButton>
    <div class="flex flex-wrap items-start gap-4">
      <span
        class="flex size-12 shrink-0 items-center justify-center rounded-lg bg-surface-control text-fg-secondary"
      >
        <component :is="icon" class="size-6" :stroke-width="1.5" />
      </span>
      <div class="min-w-0 flex-1 basis-48">
        <h2 class="text-title-sm font-semibold break-words text-fg-primary">{{ name }}</h2>
        <p v-if="description" class="mt-1 text-body text-fg-secondary">{{ description }}</p>
        <slot name="status" />
      </div>
      <div v-if="$slots.default" class="flex flex-wrap items-center gap-2"><slot /></div>
    </div>
  </header>
</template>
