<script setup lang="ts">
import { Check, ChevronDown, Search } from "@lucide/vue";
import { PopoverContent, PopoverPortal, PopoverRoot, PopoverTrigger } from "reka-ui";
import { computed, nextTick, shallowRef, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import {
  MENU_ITEM_BASE_CLASSES,
  POPOVER_SURFACE_CLASSES,
  POPUP_ANIMATE_CLASSES,
} from "../../../../shared/ui/ui-classes";
import { CODE_BLOCK_LANGUAGES, codeLanguageLabel, matchCodeLanguage } from "../tiptap-code-block-languages";

const props = defineProps<{
  language: string;
  editable: boolean;
}>();
const emit = defineEmits<{ select: [language: string] }>();
const { t } = useI18n();

const open = shallowRef(false);
const query = shallowRef("");
const searchInputRef = useTemplateRef<HTMLInputElement>("searchInputRef");

const languageText = computed(() =>
  props.language ? codeLanguageLabel(props.language) : t("spaces.richtext.codeBlock.plainText"),
);
const filteredLanguages = computed(() =>
  CODE_BLOCK_LANGUAGES.filter((item) => matchCodeLanguage(item, query.value)),
);

watch(open, async (visible) => {
  if (!visible) return;
  query.value = "";
  await nextTick();
  searchInputRef.value?.focus();
});

function pick(value: string) {
  open.value = false;
  emit("select", value);
}

function selectFirstMatch() {
  const first = filteredLanguages.value[0];
  if (first) pick(first.value);
}
</script>

<template>
  <PopoverRoot v-model:open="open">
    <PopoverTrigger as-child>
      <button
        type="button"
        data-testid="code-block-language"
        :disabled="!editable"
        class="inline-flex h-6 items-center gap-1 rounded-md px-1.5 text-caption-sm font-medium text-fg-tertiary transition-colors duration-100 ease-standard outline-none hover:bg-surface-control-hover hover:text-fg-primary focus-visible:ring-2 focus-visible:ring-stroke-focus disabled:pointer-events-none"
      >
        <span>{{ languageText }}</span>
        <ChevronDown class="size-3 opacity-60" :stroke-width="1.6" />
      </button>
    </PopoverTrigger>
    <PopoverPortal>
      <PopoverContent
        side="bottom"
        align="start"
        :side-offset="4"
        :class="[
          POPUP_ANIMATE_CLASSES,
          POPOVER_SURFACE_CLASSES,
          'z-50 w-48 overflow-hidden p-1 outline-none',
        ]"
      >
        <div class="flex items-center gap-1.5 border-b border-stroke-divider px-2 py-1.5">
          <Search class="size-3 shrink-0 text-fg-tertiary" :stroke-width="1.6" />
          <input
            ref="searchInputRef"
            v-model="query"
            data-testid="code-block-language-search"
            type="text"
            :placeholder="t('spaces.richtext.codeBlock.searchLanguage')"
            class="min-w-0 flex-1 bg-transparent text-caption text-fg-primary outline-none placeholder:text-fg-tertiary"
            @keydown.enter.prevent="selectFirstMatch"
          />
        </div>
        <div class="max-h-56 overflow-y-auto pt-1">
          <button
            v-if="!query.trim()"
            type="button"
            :class="[MENU_ITEM_BASE_CLASSES, 'w-full']"
            @click="pick('')"
          >
            <span class="min-w-0 flex-1 text-left">{{ t("spaces.richtext.codeBlock.plainText") }}</span>
            <Check v-if="!language" class="ml-auto size-3.5 text-fg-accent" :stroke-width="1.75" />
          </button>
          <button
            v-for="item in filteredLanguages"
            :key="item.value"
            type="button"
            :class="[MENU_ITEM_BASE_CLASSES, 'w-full']"
            @click="pick(item.value)"
          >
            <span class="min-w-0 flex-1 text-left">{{ item.label }}</span>
            <Check
              v-if="language === item.value"
              class="ml-auto size-3.5 text-fg-accent"
              :stroke-width="1.75"
            />
          </button>
        </div>
      </PopoverContent>
    </PopoverPortal>
  </PopoverRoot>
</template>
