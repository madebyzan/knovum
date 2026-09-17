<script setup lang="ts">
import { GitCompareArrows, ListChecks, SearchX } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";

defineProps<{ busy: boolean }>();
const emit = defineEmits<{ select: [prompt: string] }>();
const { t } = useI18n();
</script>

<template>
  <div class="mx-auto flex min-h-full w-full max-w-md flex-col justify-end px-3 py-3">
    <p class="px-1 text-micro font-medium text-foreground-tertiary">
      {{ t("spaces.assistant.aiStarters") }}
    </p>
    <div class="mt-1.5 flex flex-col gap-1">
      <UiButton
        size="sm"
        variant="ghost"
        class="w-full justify-start px-2.5"
        :disabled="busy"
        @click="emit('select', t('spaces.assistant.aiPrompt.compare'))"
      >
        <GitCompareArrows class="size-(--knovum-icon-size-xs)" />
        {{ t("spaces.assistant.aiAction.compare") }}
      </UiButton>
      <UiButton
        size="sm"
        variant="ghost"
        class="w-full justify-start px-2.5"
        :disabled="busy"
        @click="emit('select', t('spaces.assistant.aiPrompt.synthesize'))"
      >
        <ListChecks class="size-(--knovum-icon-size-xs)" />
        {{ t("spaces.assistant.aiAction.synthesize") }}
      </UiButton>
      <UiButton
        size="sm"
        variant="ghost"
        class="w-full justify-start px-2.5"
        :disabled="busy"
        @click="emit('select', t('spaces.assistant.aiPrompt.contradictions'))"
      >
        <SearchX class="size-(--knovum-icon-size-xs)" />
        {{ t("spaces.assistant.aiAction.contradictions") }}
      </UiButton>
    </div>
  </div>
</template>
