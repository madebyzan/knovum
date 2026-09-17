<script setup lang="ts">
import { Check, ChevronDown, Heading, List, ListChecks, ListOrdered, Pilcrow, Quote } from "@lucide/vue";
import type { Editor } from "@tiptap/vue-3";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiDropdownMenu from "../../../../shared/ui/UiDropdownMenu.vue";
import UiDropdownMenuItem from "../../../../shared/ui/UiDropdownMenuItem.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";

const props = defineProps<{ editor: Editor }>();
const { t } = useI18n();
const headings = [1, 2, 3, 4, 5, 6] as const;
const activeHeading = computed(() => headings.find((level) => props.editor.isActive("heading", { level })));
</script>

<template>
  <UiDropdownMenu align="start" :side-offset="5" class="min-w-48">
    <template #trigger>
      <button
        type="button"
        class="flex h-(--knovum-control-height-sm) min-w-14 items-center justify-center gap-1 rounded-control px-1.5 text-foreground-secondary hover:bg-control-hover hover:text-foreground data-[state=open]:bg-control-pressed"
        :aria-label="t('editor.headingMenu')"
        :title="t('editor.headingMenu')"
      >
        <Heading class="size-(--knovum-icon-size-sm)" :stroke-width="1.6" />
        <span v-if="activeHeading" class="text-caption font-semibold">{{ activeHeading }}</span>
        <ChevronDown class="size-(--knovum-icon-size-xs)" :stroke-width="1.6" />
      </button>
    </template>

    <UiDropdownMenuItem @select="editor.chain().focus().setParagraph().run()">
      <Pilcrow class="size-(--knovum-icon-size-sm)" />
      <span class="min-w-0 flex-1 truncate">{{ t("editor.paragraph") }}</span>
      <Check v-if="editor.isActive('paragraph')" class="ml-auto size-(--knovum-icon-size-xs) text-accent" />
    </UiDropdownMenuItem>
    <UiDropdownMenuItem
      v-for="level in headings"
      :key="level"
      @select="editor.chain().focus().toggleHeading({ level }).run()"
    >
      <span class="w-5 font-semibold">H{{ level }}</span>
      <span class="min-w-0 flex-1 truncate">{{ t(`editor.heading${level}`) }}</span>
      <Check
        v-if="editor.isActive('heading', { level })"
        class="ml-auto size-(--knovum-icon-size-xs) text-accent"
      />
    </UiDropdownMenuItem>
  </UiDropdownMenu>

  <UiDropdownMenu align="start" :side-offset="5" class="min-w-48">
    <template #trigger>
      <button
        type="button"
        class="flex h-(--knovum-control-height-sm) min-w-12 items-center justify-center gap-0.5 rounded-control px-1.5 text-foreground-secondary hover:bg-control-hover hover:text-foreground data-[state=open]:bg-control-pressed"
        :aria-label="t('editor.listMenu')"
        :title="t('editor.listMenu')"
      >
        <List class="size-(--knovum-icon-size-sm)" :stroke-width="1.6" />
        <ChevronDown class="size-(--knovum-icon-size-xs)" />
      </button>
    </template>

    <UiDropdownMenuItem @select="editor.chain().focus().toggleBulletList().run()">
      <List class="size-(--knovum-icon-size-sm)" />{{ t("editor.bulletList") }}
    </UiDropdownMenuItem>
    <UiDropdownMenuItem @select="editor.chain().focus().toggleOrderedList().run()">
      <ListOrdered class="size-(--knovum-icon-size-sm)" />{{ t("editor.orderedList") }}
    </UiDropdownMenuItem>
    <UiDropdownMenuItem @select="editor.chain().focus().toggleTaskList().run()">
      <ListChecks class="size-(--knovum-icon-size-sm)" />{{ t("editor.taskList") }}
    </UiDropdownMenuItem>
  </UiDropdownMenu>

  <UiIconButton
    :label="t('editor.blockquote')"
    size="sm"
    :active="editor.isActive('blockquote')"
    @click="editor.chain().focus().toggleBlockquote().run()"
  >
    <Quote class="size-(--knovum-icon-size-sm)" :stroke-width="1.6" />
  </UiIconButton>
</template>
