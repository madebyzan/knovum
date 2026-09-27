<script setup lang="ts">
import UiButton from "../../../../shared/ui/UiButton.vue";
import type { AssistantPermissionOption } from "../../contracts/generated/AssistantPermissionOption";

defineProps<{
  title: string;
  options: readonly AssistantPermissionOption[];
  busy: boolean;
}>();
const emit = defineEmits<{ decide: [optionId: string | null] }>();
</script>

<template>
  <section
    class="border-l-border-warning mb-4 rounded-panel border border-l-2 border-stroke-default bg-surface-muted p-3 text-caption-md"
    role="group"
  >
    <h3 class="font-semibold text-fg-primary">{{ $t("spaces.assistant.permission.request") }}</h3>
    <p class="mt-1 text-fg-secondary">{{ title }}</p>
    <div class="mt-3 flex flex-wrap justify-end gap-2">
      <UiButton
        v-for="option in options"
        :key="option.id"
        size="sm"
        variant="secondary"
        :disabled="busy"
        @click="emit('decide', option.id)"
      >
        {{ option.label }}
      </UiButton>
      <UiButton size="sm" variant="ghost" :disabled="busy" @click="emit('decide', null)">
        {{ $t("spaces.assistant.permission.deny") }}
      </UiButton>
    </div>
  </section>
</template>
