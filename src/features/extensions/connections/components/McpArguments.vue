<script setup lang="ts">
import { Plus, Trash2 } from "@lucide/vue";
import { useId } from "vue";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
const args = defineModel<string[]>({ required: true });
defineProps<{ disabled?: boolean }>();
const headingId = useId();
</script>
<template>
  <section class="space-y-2" :aria-labelledby="headingId">
    <header class="flex items-center justify-between gap-2">
      <h3 :id="headingId" class="text-label font-medium">
        {{ $t("extensions.endpoint.process.arguments.label") }}
      </h3>
      <UiButton
        size="sm"
        variant="ghost"
        :disabled="disabled"
        data-testid="add-mcp-argument"
        @click="args.push('')"
      >
        <Plus class="size-3.5" />{{ $t("extensions.endpoint.process.arguments.add") }}
      </UiButton>
    </header>
    <div v-for="(argument, index) in args" :key="index" class="flex items-center gap-2">
      <UiInput
        :model-value="argument"
        @update:model-value="args[index] = $event"
        :disabled="disabled"
        class="min-w-0 font-mono text-caption"
        :aria-label="$t('extensions.endpoint.process.arguments.at', { index: index + 1 })"
        :data-testid="`mcp-argument-${index}`"
      />
      <UiIconButton
        size="sm"
        :disabled="disabled"
        :label="$t('extensions.endpoint.process.arguments.remove', { index: index + 1 })"
        @click="args.splice(index, 1)"
        ><Trash2 class="size-3.5"
      /></UiIconButton>
    </div>
    <p class="text-caption text-fg-secondary">{{ $t("extensions.endpoint.process.arguments.hint") }}</p>
  </section>
</template>
