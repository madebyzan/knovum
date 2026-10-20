<script setup lang="ts">
import { Check, ChevronDown } from "@lucide/vue";
import type { Editor } from "@tiptap/vue-3";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiDropdownMenu from "../../../../shared/ui/UiDropdownMenu.vue";
import UiDropdownMenuItem from "../../../../shared/ui/UiDropdownMenuItem.vue";
import { LABEL_KEY_BY_HEADING_LEVEL, HEADING_LEVELS } from "../heading-levels";

const props = defineProps<{ editor: Editor }>();
const { t } = useI18n();
const activeHeading = computed(() =>
  HEADING_LEVELS.find((level) => props.editor.isActive("heading", { level })),
);
const currentLabel = computed(() =>
  activeHeading.value ? `H${activeHeading.value}` : t("spaces.richtext.paragraph"),
);
</script>

<template>
  <UiDropdownMenu align="start" :side-offset="6" class="min-w-52">
    <template #trigger>
      <button
        type="button"
        class="flex h-(--knovum-control-height-sm) min-w-16 items-center justify-center gap-1 rounded-md px-2 text-label text-fg-secondary outline-none hover:bg-surface-control-hover hover:text-fg-primary data-[state=open]:bg-surface-control"
        :aria-label="t('spaces.richtext.headings.menu')"
      >
        <span class="max-w-20 truncate">{{ currentLabel }}</span>
        <ChevronDown class="size-3 text-fg-tertiary" :stroke-width="1.6" />
      </button>
    </template>

    <UiDropdownMenuItem @select="editor.chain().focus().setParagraph().run()">
      <span class="w-6 text-caption font-medium">Aa</span>
      <span class="min-w-0 flex-1 truncate">{{ t("spaces.richtext.paragraph") }}</span>
      <Check v-if="editor.isActive('paragraph')" class="ml-auto size-3 text-fg-accent" />
    </UiDropdownMenuItem>
    <UiDropdownMenuItem
      v-for="level in HEADING_LEVELS"
      :key="level"
      @select="editor.chain().focus().toggleHeading({ level }).run()"
    >
      <span class="w-6 text-caption font-semibold">H{{ level }}</span>
      <span class="min-w-0 flex-1 truncate">{{ t(LABEL_KEY_BY_HEADING_LEVEL[level]) }}</span>
      <Check v-if="editor.isActive('heading', { level })" class="ml-auto size-3 text-fg-accent" />
    </UiDropdownMenuItem>
  </UiDropdownMenu>
</template>
