<script setup lang="ts">
import { Check } from "@lucide/vue";
import { computed } from "vue";

import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiPopover from "../../../../shared/ui/UiPopover.vue";

interface CanvasColorOption {
  value: string;
  label: string;
  color: string;
}

const props = defineProps<{
  label: string;
  options: readonly CanvasColorOption[];
  selected: string;
  columns?: 6;
}>();
const emit = defineEmits<{ select: [value: string] }>();
const selectedColor = computed(() => props.options.find(({ value }) => value === props.selected)?.color);
</script>

<template>
  <UiPopover side="top" align="center" :side-offset="5" class="p-0.5">
    <template #trigger>
      <UiIconButton :label="label" size="sm">
        <span class="size-4 rounded-full ring-1 ring-border" :style="{ backgroundColor: selectedColor }" />
      </UiIconButton>
    </template>
    <div class="gap-0.5" :class="columns === 6 ? 'grid grid-cols-6' : 'flex items-center'">
      <button
        v-for="option in options"
        :key="option.value"
        type="button"
        class="grid size-5 place-items-center rounded-control transition-colors duration-100 outline-none hover:bg-control-hover focus-visible:ring-2 focus-visible:ring-focus-ring"
        :aria-label="option.label"
        :aria-pressed="selected === option.value"
        @click="emit('select', option.value)"
      >
        <span
          class="grid size-4 place-items-center rounded-full ring-1 ring-border"
          :style="{ backgroundColor: option.color }"
        >
          <Check
            v-if="selected === option.value"
            class="size-3 text-white drop-shadow-sm"
            :stroke-width="2.5"
          />
        </span>
      </button>
    </div>
  </UiPopover>
</template>
