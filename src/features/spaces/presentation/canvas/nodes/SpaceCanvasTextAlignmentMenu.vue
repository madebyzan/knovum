<script setup lang="ts">
import { AlignCenter, AlignLeft, AlignRight, Check } from "@lucide/vue";
import type { Editor } from "@tiptap/vue-3";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiDropdownMenu from "../../../../../shared/ui/UiDropdownMenu.vue";
import UiDropdownMenuItem from "../../../../../shared/ui/UiDropdownMenuItem.vue";
import UiIconButton from "../../../../../shared/ui/UiIconButton.vue";

const props = defineProps<{
  editor: Editor;
  selected: "left" | "center" | "right";
}>();
const { t } = useI18n();
const alignments = [
  { value: "left", icon: AlignLeft },
  { value: "center", icon: AlignCenter },
  { value: "right", icon: AlignRight },
] as const;
const selectedAlignment = computed(() => alignments.find(({ value }) => value === props.selected)!);
</script>

<template>
  <UiDropdownMenu side="top" align="end" :side-offset="5" :modal="false" class="min-w-32">
    <template #trigger>
      <UiIconButton :label="t(`spaces.canvas.textAlignments.${selected}`)" size="sm">
        <component :is="selectedAlignment.icon" class="size-(--knovum-icon-size-sm)" :stroke-width="1.7" />
      </UiIconButton>
    </template>
    <UiDropdownMenuItem
      v-for="alignment in alignments"
      :key="alignment.value"
      @select="editor.chain().focus().setTextAlign(alignment.value).run()"
    >
      <component :is="alignment.icon" class="size-(--knovum-icon-size-sm)" :stroke-width="1.7" />
      {{ t(`spaces.canvas.textAlignments.${alignment.value}`) }}
      <Check
        class="ml-auto size-(--knovum-icon-size-xs)"
        :class="selected === alignment.value ? 'opacity-100' : 'opacity-0'"
      />
    </UiDropdownMenuItem>
  </UiDropdownMenu>
</template>
