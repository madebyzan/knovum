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
const selectedItem = computed(() => alignments.find(({ value }) => value === props.selected)!);
</script>

<template>
  <UiDropdownMenu side="top" align="end" :side-offset="5" :modal="false" class="min-w-32">
    <template #trigger>
      <UiIconButton :label="t(`spaces.canvas.textAlignments.${selected}`)" size="sm">
        <component :is="selectedItem.icon" class="size-(--knovum-icon-size-sm)" :stroke-width="1.7" />
      </UiIconButton>
    </template>
    <UiDropdownMenuItem
      v-for="item in alignments"
      :key="item.value"
      @select="editor.chain().focus().setTextAlign(item.value).run()"
    >
      <component :is="item.icon" class="size-(--knovum-icon-size-sm)" :stroke-width="1.7" />
      {{ t(`spaces.canvas.textAlignments.${item.value}`) }}
      <Check
        class="ml-auto size-(--knovum-icon-size-xs)"
        :class="selected === item.value ? 'opacity-100' : 'opacity-0'"
      />
    </UiDropdownMenuItem>
  </UiDropdownMenu>
</template>
