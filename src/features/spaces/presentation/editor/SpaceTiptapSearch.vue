<script setup lang="ts">
import { ChevronDown, ChevronRight, ChevronUp, ReplaceAll, ReplaceIcon, Search, X } from "@lucide/vue";
import type { Editor } from "@tiptap/vue-3";
import { nextTick, ref, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiIconButton from "../../../../shared/ui/UiIconButton.vue";

const props = defineProps<{ editor: Editor }>();
const { t } = useI18n();
const open = ref(false);
const showReplace = ref(false);
const query = ref("");
const replaceTerm = ref("");
const searchInput = useTemplateRef<HTMLInputElement>("searchInput");
const replaceInput = useTemplateRef<HTMLInputElement>("replaceInput");

watch(query, (value) => {
  if (value) props.editor.commands.setSearchTerm(value);
  else props.editor.commands.clearSearch();
});
watch(replaceTerm, (value) => {
  props.editor.commands.setReplaceTerm(value);
});
watch(open, async (value) => {
  if (value) await nextTick(() => searchInput.value?.focus());
  else {
    query.value = "";
    replaceTerm.value = "";
    showReplace.value = false;
    props.editor.commands.clearSearch();
  }
});
function move(direction: "next" | "previous") {
  if (!query.value) return;
  if (direction === "next") props.editor.commands.goToNextResult();
  else props.editor.commands.goToPreviousResult();
}
function doReplace() {
  if (!query.value) return;
  props.editor.commands.replace();
  move("next");
}
function doReplaceAll() {
  if (!query.value) return;
  props.editor.commands.replaceAll();
}
</script>

<template>
  <div class="relative flex items-center">
    <UiIconButton :label="t('spaces.richtext.find.title')" size="sm" :active="open" @click="open = !open">
      <Search class="size-3.5" :stroke-width="1.6" />
    </UiIconButton>
    <div
      v-if="open"
      class="absolute top-9 right-0 z-50 flex flex-col gap-1.5 rounded-panel border border-stroke-default bg-surface-overlay p-1.5 shadow-popover"
    >
      <div class="flex w-72 items-center gap-1">
        <Search class="ml-1 size-3.5 shrink-0 text-fg-tertiary" :stroke-width="1.5" />
        <input
          ref="searchInput"
          v-model="query"
          class="min-w-0 flex-1 bg-transparent px-1 text-label text-fg-primary outline-none placeholder:text-fg-tertiary"
          :placeholder="t('spaces.richtext.find.placeholder')"
          @keydown.enter.prevent="move($event.shiftKey ? 'previous' : 'next')"
          @keydown.esc.prevent="open = false"
        />
        <UiIconButton
          :label="t('spaces.richtext.find.toggleReplace')"
          size="sm"
          :active="showReplace"
          @click="
            showReplace = !showReplace;
            if (showReplace) nextTick(() => replaceInput?.focus());
          "
        >
          <ChevronDown v-if="showReplace" class="size-3" />
          <ChevronRight v-else class="size-3" />
        </UiIconButton>
        <UiIconButton
          :label="t('spaces.richtext.find.previous')"
          size="sm"
          :disabled="!query"
          @click="move('previous')"
        >
          <ChevronUp class="size-3" />
        </UiIconButton>
        <UiIconButton
          :label="t('spaces.richtext.find.next')"
          size="sm"
          :disabled="!query"
          @click="move('next')"
        >
          <ChevronDown class="size-3" />
        </UiIconButton>
        <UiIconButton :label="t('common.close')" size="sm" @click="open = false">
          <X class="size-3" />
        </UiIconButton>
      </div>
      <div v-if="showReplace" class="flex w-72 items-center gap-1">
        <ReplaceIcon class="ml-1 size-3.5 shrink-0 text-fg-tertiary" :stroke-width="1.5" />
        <input
          ref="replaceInput"
          v-model="replaceTerm"
          class="min-w-0 flex-1 bg-transparent px-1 text-label text-fg-primary outline-none placeholder:text-fg-tertiary"
          :placeholder="t('spaces.richtext.find.replacePlaceholder')"
          @keydown.enter.prevent="doReplace"
          @keydown.esc.prevent="open = false"
        />
        <UiIconButton
          :label="t('spaces.richtext.find.replace')"
          size="sm"
          :disabled="!query"
          @click="doReplace"
        >
          <ReplaceIcon class="size-3" />
        </UiIconButton>
        <UiIconButton
          :label="t('spaces.richtext.find.replaceAll')"
          size="sm"
          :disabled="!query"
          @click="doReplaceAll"
        >
          <ReplaceAll class="size-3" />
        </UiIconButton>
      </div>
    </div>
  </div>
</template>
