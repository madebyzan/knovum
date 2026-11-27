<script setup lang="ts">
import { ChevronDown, ChevronRight, ChevronUp, ReplaceAll, ReplaceIcon, Search, X } from "@lucide/vue";
import type { Editor } from "@tiptap/vue-3";
import { useTemplateRef } from "vue";
import { useI18n } from "vue-i18n";

import { POPOVER_SURFACE_CLASSES } from "../../../../shared/ui/ui-classes";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import { useSpaceTiptapSearch } from "./use-space-tiptap-search";

const props = defineProps<{ editor: Editor }>();
const { t } = useI18n();
const searchInput = useTemplateRef<HTMLInputElement>("searchInput");
const replaceInput = useTemplateRef<HTMLInputElement>("replaceInput");
const search = useSpaceTiptapSearch(() => props.editor, { search: searchInput, replace: replaceInput });
const { open, showReplace, query, replaceTerm, matchTotal, matchSummary } = search;

defineExpose({ show: search.show });
</script>

<template>
  <div class="relative flex items-center">
    <UiIconButton :label="t('spaces.richtext.find.title')" size="sm" :active="open" @click="open = !open">
      <Search class="size-3.5" :stroke-width="1.6" />
    </UiIconButton>
    <div
      v-if="open"
      data-testid="tiptap-search-popover"
      :class="[POPOVER_SURFACE_CLASSES, 'absolute top-9 right-0 z-50 flex flex-col gap-1.5 p-1.5']"
    >
      <div class="flex w-80 items-center gap-1">
        <Search class="ml-1 size-3.5 shrink-0 text-fg-tertiary" :stroke-width="1.5" />
        <input
          ref="searchInput"
          v-model="query"
          data-testid="tiptap-search-input"
          class="min-w-0 flex-1 bg-transparent px-1 text-label text-fg-primary outline-none placeholder:text-fg-tertiary"
          :placeholder="t('spaces.richtext.find.placeholder')"
          @keydown="search.handleSearchKeydown"
        />
        <span
          v-if="matchSummary"
          data-testid="tiptap-search-count"
          class="shrink-0 px-1 text-caption-sm tabular-nums"
          :class="matchTotal === 0 ? 'text-fg-danger' : 'text-fg-tertiary'"
        >
          {{ matchSummary }}
        </span>
        <UiIconButton
          :label="t('spaces.richtext.find.toggleReplace')"
          tooltip
          size="sm"
          :active="showReplace"
          @click="search.toggleReplace"
        >
          <ChevronDown v-if="showReplace" class="size-3" />
          <ChevronRight v-else class="size-3" />
        </UiIconButton>
        <UiIconButton
          :label="t('spaces.richtext.find.previous')"
          size="sm"
          :disabled="!query || matchTotal === 0"
          @click="search.move('previous')"
        >
          <ChevronUp class="size-3" />
        </UiIconButton>
        <UiIconButton
          :label="t('spaces.richtext.find.next')"
          size="sm"
          :disabled="!query || matchTotal === 0"
          @click="search.move('next')"
        >
          <ChevronDown class="size-3" />
        </UiIconButton>
        <UiIconButton :label="t('common.close')" size="sm" @click="open = false">
          <X class="size-3" />
        </UiIconButton>
      </div>
      <div v-if="showReplace" class="flex w-80 items-center gap-1">
        <ReplaceIcon class="ml-1 size-3.5 shrink-0 text-fg-tertiary" :stroke-width="1.5" />
        <input
          ref="replaceInput"
          v-model="replaceTerm"
          class="min-w-0 flex-1 bg-transparent px-1 text-label text-fg-primary outline-none placeholder:text-fg-tertiary"
          :placeholder="t('spaces.richtext.find.replace.placeholder')"
          @keydown.enter.prevent="search.replaceMatch"
          @keydown.esc.prevent="open = false"
        />
        <UiIconButton
          :label="t('spaces.richtext.find.replace.action')"
          tooltip
          size="sm"
          :disabled="!query || matchTotal === 0"
          @click="search.replaceMatch"
        >
          <ReplaceIcon class="size-3" />
        </UiIconButton>
        <UiIconButton
          :label="t('spaces.richtext.find.replaceAll')"
          tooltip
          size="sm"
          :disabled="!query || matchTotal === 0"
          @click="search.replaceAllMatches"
        >
          <ReplaceAll class="size-3" />
        </UiIconButton>
      </div>
    </div>
  </div>
</template>
