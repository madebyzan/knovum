<script setup lang="ts">
import {
  Check,
  ChevronDown,
  Code2,
  Heading1,
  Heading2,
  Heading3,
  List,
  ListChecks,
  ListOrdered,
  Pilcrow,
  Quote,
} from "@lucide/vue";
import type { Editor } from "@tiptap/vue-3";
import { computed, type Component } from "vue";
import { useI18n } from "vue-i18n";

import UiDropdownMenu from "../../../../shared/ui/UiDropdownMenu.vue";
import UiDropdownMenuItem from "../../../../shared/ui/UiDropdownMenuItem.vue";
import { CalloutBlockIcon } from "../editor-icons";

const props = defineProps<{ editor: Editor }>();
const { t } = useI18n();

interface TurnIntoOption {
  readonly id: string;
  readonly labelKey: string;
  readonly icon: Component;
  readonly active: (editor: Editor) => boolean;
  readonly apply: (editor: Editor) => void;
}

const OPTIONS: readonly TurnIntoOption[] = [
  {
    id: "paragraph",
    labelKey: "spaces.richtext.paragraph",
    icon: Pilcrow,
    active: (editor) => editor.isActive("paragraph") && !editor.isActive("callout"),
    apply: (editor) => {
      const chain = editor.chain().focus();
      if (editor.isActive("callout")) chain.lift("callout");
      chain.setParagraph().run();
    },
  },
  {
    id: "h1",
    labelKey: "spaces.richtext.heading1",
    icon: Heading1,
    active: (editor) => editor.isActive("heading", { level: 1 }),
    apply: (editor) => void editor.chain().focus().toggleHeading({ level: 1 }).run(),
  },
  {
    id: "h2",
    labelKey: "spaces.richtext.heading2",
    icon: Heading2,
    active: (editor) => editor.isActive("heading", { level: 2 }),
    apply: (editor) => void editor.chain().focus().toggleHeading({ level: 2 }).run(),
  },
  {
    id: "h3",
    labelKey: "spaces.richtext.heading3",
    icon: Heading3,
    active: (editor) => editor.isActive("heading", { level: 3 }),
    apply: (editor) => void editor.chain().focus().toggleHeading({ level: 3 }).run(),
  },
  {
    id: "bullet",
    labelKey: "spaces.richtext.bulletList",
    icon: List,
    active: (editor) => editor.isActive("bulletList"),
    apply: (editor) => void editor.chain().focus().toggleBulletList().run(),
  },
  {
    id: "ordered",
    labelKey: "spaces.richtext.orderedList",
    icon: ListOrdered,
    active: (editor) => editor.isActive("orderedList"),
    apply: (editor) => void editor.chain().focus().toggleOrderedList().run(),
  },
  {
    id: "task",
    labelKey: "spaces.richtext.taskList",
    icon: ListChecks,
    active: (editor) => editor.isActive("taskList"),
    apply: (editor) => void editor.chain().focus().toggleTaskList().run(),
  },
  {
    id: "blockquote",
    labelKey: "spaces.richtext.blockquote",
    icon: Quote,
    active: (editor) => editor.isActive("blockquote"),
    apply: (editor) => void editor.chain().focus().toggleBlockquote().run(),
  },
  {
    id: "callout",
    labelKey: "spaces.richtext.callout.title",
    icon: CalloutBlockIcon,
    active: (editor) => editor.isActive("callout"),
    apply: (editor) => void editor.chain().focus().toggleCallout("note").run(),
  },
  {
    id: "code",
    labelKey: "spaces.richtext.codeBlock.title",
    icon: Code2,
    active: (editor) => editor.isActive("codeBlock"),
    apply: (editor) => void editor.chain().focus().toggleCodeBlock().run(),
  },
];

const currentLabel = computed(() => {
  const matched = OPTIONS.find((option) => option.active(props.editor));
  return t(matched?.labelKey ?? "spaces.richtext.paragraph");
});
</script>

<template>
  <UiDropdownMenu align="start" :side-offset="6" class="min-w-48">
    <template #trigger>
      <button
        type="button"
        class="flex h-(--knovum-control-height-sm) items-center justify-center gap-1 rounded-md px-2 text-label text-fg-secondary transition-colors outline-none hover:bg-surface-control-hover hover:text-fg-primary data-[state=open]:bg-surface-control"
        :aria-label="t('spaces.richtext.blockMenu.turnInto')"
      >
        <span class="max-w-24 truncate">{{ currentLabel }}</span>
        <ChevronDown class="size-3 text-fg-tertiary" :stroke-width="1.65" />
      </button>
    </template>

    <UiDropdownMenuItem v-for="option in OPTIONS" :key="option.id" @select="option.apply(editor)">
      <component :is="option.icon" class="size-3.5 text-fg-secondary" :stroke-width="1.65" />
      <span class="min-w-0 flex-1 truncate">{{ t(option.labelKey) }}</span>
      <Check v-if="option.active(editor)" class="ml-auto size-3 text-fg-accent" />
    </UiDropdownMenuItem>
  </UiDropdownMenu>
</template>
