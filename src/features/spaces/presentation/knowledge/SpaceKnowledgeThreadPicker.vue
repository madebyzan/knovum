<script setup lang="ts">
import { ChevronDown, History } from "@lucide/vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiDropdownMenu from "../../../../shared/ui/UiDropdownMenu.vue";
import UiDropdownMenuItem from "../../../../shared/ui/UiDropdownMenuItem.vue";

const modelValue = defineModel<string>({ required: true });
const props = defineProps<{
  options: readonly { value: string; label: string }[];
  disabled?: boolean;
}>();
const { t } = useI18n();
const activeLabel = computed(
  () =>
    props.options.find((option) => option.value === modelValue.value)?.label ??
    t("spaces.assistant.newKnowledgeThread"),
);
</script>

<template>
  <UiDropdownMenu align="start" :side-offset="6" class="max-h-72 max-w-80 min-w-56 overflow-y-auto">
    <template #trigger>
      <button
        type="button"
        class="flex h-(--knovum-control-height-sm) min-w-0 flex-1 items-center gap-1.5 rounded-control px-2 text-caption text-foreground-secondary outline-none hover:bg-control-hover hover:text-foreground disabled:pointer-events-none disabled:opacity-40"
        :disabled="disabled"
      >
        <History class="size-(--knovum-icon-size-xs) shrink-0" :stroke-width="1.55" />
        <span class="min-w-0 flex-1 truncate text-left">{{ activeLabel }}</span>
        <ChevronDown
          class="size-(--knovum-icon-size-xs) shrink-0 text-foreground-tertiary"
          :stroke-width="1.55"
        />
      </button>
    </template>

    <UiDropdownMenuItem v-for="option in options" :key="option.value" @select="modelValue = option.value">
      <span class="min-w-0 flex-1 truncate">{{ option.label }}</span>
    </UiDropdownMenuItem>
  </UiDropdownMenu>
</template>
