<script setup lang="ts">
import { Check, Code2, Download, Eye, MessageSquareQuote, MoreHorizontal } from "@lucide/vue";
import type { Editor } from "@tiptap/vue-3";
import { computed, useTemplateRef } from "vue";
import { useI18n } from "vue-i18n";

import UiDropdownMenu from "../../../../shared/ui/UiDropdownMenu.vue";
import UiDropdownMenuItem from "../../../../shared/ui/UiDropdownMenuItem.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiKbd from "../../../../shared/ui/UiKbd.vue";
import UiKnovumAiIcon from "../../../../shared/ui/UiKnovumAiIcon.vue";
import UiSeparator from "../../../../shared/ui/UiSeparator.vue";
import { nodeIcon } from "../../canvas/presentation/node-icon";
import SpaceEditorSaveStatus from "../../workspace/presentation/SpaceEditorSaveStatus.vue";
import type { EditorSaveStatus } from "../../workspace/use-editor-autosave";
import { FormatToolbarIcon, PageWidthIcon } from "../editor-icons";
import SpaceTiptapSearch from "./SpaceTiptapSearch.vue";

const props = defineProps<{
  kind: "note" | "markdown";
  editorLabel: string;
  saveStatus: EditorSaveStatus;
  editor: Editor | undefined;
  active: boolean;
  showToolbar: boolean;
  fullWidth: boolean;
  sourceMode: boolean;
  scrolled: boolean;
  wordCount: number;
  characterCount: number;
}>();

const emit = defineEmits<{
  openAi: [];
  askInChat: [];
  toggleToolbar: [];
  toggleFullWidth: [];
  toggleSourceMode: [];
  exportMarkdown: [];
}>();

const { t } = useI18n();
const search = useTemplateRef<InstanceType<typeof SpaceTiptapSearch>>("search");
const readingMinutes = computed(() =>
  props.wordCount > 0 ? Math.max(1, Math.ceil(props.wordCount / 220)) : 0,
);

defineExpose({
  showSearch: () => search.value?.show(),
});
</script>

<template>
  <header
    class="relative z-20 flex h-toolbar shrink-0 items-center gap-2 border-b bg-surface-canvas px-4 transition-colors duration-100 ease-standard"
    :class="scrolled ? 'border-stroke-divider' : 'border-transparent'"
  >
    <component :is="nodeIcon(kind)" class="size-3.5 text-fg-tertiary" :stroke-width="1.6" />
    <span class="text-caption font-medium text-fg-secondary">{{ editorLabel }}</span>
    <SpaceEditorSaveStatus :status="saveStatus" />
    <div class="flex-1" />
    <span
      v-if="wordCount || characterCount"
      class="hidden text-caption-sm text-fg-tertiary tabular-nums sm:block"
    >
      {{ t("spaces.richtext.statistics.words", { count: wordCount }, wordCount)
      }}<template v-if="readingMinutes">
        · {{ t("spaces.richtext.statistics.readingMinutes", { count: readingMinutes }, readingMinutes) }}
      </template>
    </span>
    <SpaceTiptapSearch ref="search" v-if="editor && active && !sourceMode" :editor="editor" />
    <UiDropdownMenu align="end" :side-offset="6" class="w-56">
      <template #trigger>
        <UiIconButton
          data-testid="editor-page-menu"
          :label="t('spaces.richtext.modes.pageOptions')"
          size="sm"
        >
          <MoreHorizontal class="size-4" :stroke-width="1.6" />
        </UiIconButton>
      </template>
      <UiDropdownMenuItem v-if="editor && !sourceMode" @select="emit('openAi')">
        <UiKnovumAiIcon class="size-3.5 text-fg-secondary" />
        <span class="min-w-0 flex-1">{{ t("spaces.richtext.ai.panel.ask") }}</span>
        <UiKbd shortcut="$mod+J" />
      </UiDropdownMenuItem>
      <UiDropdownMenuItem @select="emit('askInChat')">
        <MessageSquareQuote class="size-3.5 text-fg-secondary" :stroke-width="1.6" />
        <span class="min-w-0 flex-1">{{ t("spaces.richtext.blockMenu.askInChat") }}</span>
      </UiDropdownMenuItem>
      <UiSeparator class="my-1" />
      <UiDropdownMenuItem v-if="!sourceMode" @select="emit('toggleToolbar')">
        <FormatToolbarIcon class="size-3.5 text-fg-secondary" :stroke-width="1.65" />
        <span class="min-w-0 flex-1">{{ t("spaces.richtext.modes.formattingBar") }}</span>
        <Check v-if="showToolbar" class="ml-auto size-3.5 text-fg-accent" :stroke-width="1.75" />
      </UiDropdownMenuItem>
      <UiDropdownMenuItem @select="emit('toggleFullWidth')">
        <PageWidthIcon class="size-3.5 text-fg-secondary" :stroke-width="1.65" />
        <span class="min-w-0 flex-1">{{ t("spaces.richtext.modes.fullWidth") }}</span>
        <Check v-if="fullWidth" class="ml-auto size-3.5 text-fg-accent" :stroke-width="1.75" />
      </UiDropdownMenuItem>
      <UiDropdownMenuItem v-if="kind === 'markdown'" @select="emit('toggleSourceMode')">
        <Eye v-if="sourceMode" class="size-3.5 text-fg-secondary" :stroke-width="1.6" />
        <Code2 v-else class="size-3.5 text-fg-secondary" :stroke-width="1.6" />
        <span class="min-w-0 flex-1">{{
          sourceMode ? t("spaces.richtext.modes.rich") : t("spaces.richtext.modes.source")
        }}</span>
      </UiDropdownMenuItem>
      <UiSeparator class="my-1" />
      <UiDropdownMenuItem @select="emit('exportMarkdown')">
        <Download class="size-3.5 text-fg-secondary" :stroke-width="1.6" />
        <span class="min-w-0 flex-1">{{ t("spaces.importExport.markdown.export") }}</span>
      </UiDropdownMenuItem>
    </UiDropdownMenu>
  </header>
</template>
