<script setup lang="ts">
import { Check, ChevronDown } from "@lucide/vue";
import type { Editor } from "@tiptap/vue-3";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiDropdownMenu from "../../../../shared/ui/UiDropdownMenu.vue";
import UiDropdownMenuItem from "../../../../shared/ui/UiDropdownMenuItem.vue";

const props = defineProps<{ editor: Editor }>();
const { t } = useI18n();
const headings = [1, 2, 3, 4, 5, 6] as const;
const activeHeading = computed(() => headings.find((level) => props.editor.isActive("heading", { level })));
const currentLabel = computed(() =>
  activeHeading.value ? `H${activeHeading.value}` : t("editor.paragraph"),
);
</script>

<template>
  <UiDropdownMenu align="start" :side-offset="6" class="min-w-52">
    <template #trigger>
      <button
        type="button"
        class="flex h-(--knovum-control-height-sm) min-w-16 items-center justify-center gap-1 rounded-control px-2 text-label text-foreground-secondary outline-none hover:bg-control-hover hover:text-foreground data-[state=open]:bg-control-pressed"
        :aria-label="t('editor.headingMenu')"
      >
        <span class="max-w-20 truncate">{{ currentLabel }}</span>
        <ChevronDown class="size-(--knovum-icon-size-xs) text-foreground-tertiary" :stroke-width="1.6" />
      </button>
    </template>

    <UiDropdownMenuItem @select="editor.chain().focus().setParagraph().run()">
      <span class="w-6 text-caption font-medium">Aa</span>
      <span class="min-w-0 flex-1 truncate">{{ t("editor.paragraph") }}</span>
      <Check v-if="editor.isActive('paragraph')" class="ml-auto size-(--knovum-icon-size-xs) text-accent" />
    </UiDropdownMenuItem>
    <UiDropdownMenuItem
      v-for="level in headings"
      :key="level"
      @select="editor.chain().focus().toggleHeading({ level }).run()"
    >
      <span class="w-6 text-caption font-semibold">H{{ level }}</span>
      <span class="min-w-0 flex-1 truncate">{{ t(`editor.heading${level}`) }}</span>
      <Check
        v-if="editor.isActive('heading', { level })"
        class="ml-auto size-(--knovum-icon-size-xs) text-accent"
      />
    </UiDropdownMenuItem>
  </UiDropdownMenu>
</template>
