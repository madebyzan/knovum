<script setup lang="ts">
import { X } from "@lucide/vue";
import { whenever } from "@vueuse/core";
import { groupBy } from "es-toolkit";
import { search } from "fast-fuzzy";
import { DialogClose, DialogContent, DialogOverlay, DialogPortal, DialogRoot, DialogTitle } from "reka-ui";
import { computed, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import type { AppCommand, CommandCategory } from "../../shared/contracts/app-command";
import UiIconButton from "../../shared/ui/UiIconButton.vue";
import UiKbd from "../../shared/ui/UiKbd.vue";
import UiSearchInput from "../../shared/ui/UiSearchInput.vue";

interface ShortcutListing {
  id: string;
  title: string;
  shortcut: string;
  category: CommandCategory;
}

const CATEGORY_ORDER: readonly CommandCategory[] = ["general", "navigation", "canvas", "editing"];

const CATEGORY_LOCALE_KEYS: Readonly<Record<CommandCategory, string>> = {
  general: "shell.shortcuts.categories.general",
  navigation: "shell.shortcuts.categories.navigation",
  canvas: "shell.shortcuts.categories.canvas",
  editing: "shell.shortcuts.categories.editing",
};

const open = defineModel<boolean>("open", { required: true });
const props = defineProps<{ commands: readonly AppCommand[] }>();
const { t } = useI18n();

const query = shallowRef("");
whenever(
  () => !open.value,
  () => {
    query.value = "";
  },
);

const allShortcuts = computed<ShortcutListing[]>(() => {
  const list: ShortcutListing[] = [];
  const listedGroups = new Set<string>();
  for (const cmd of props.commands) {
    if (!cmd.shortcut) continue;
    if (cmd.shortcutListing) {
      if (!listedGroups.has(cmd.shortcutListing.groupId)) {
        listedGroups.add(cmd.shortcutListing.groupId);
        list.push({
          id: cmd.shortcutListing.groupId,
          title: cmd.shortcutListing.title,
          shortcut: cmd.shortcutListing.shortcut,
          category: cmd.category ?? "general",
        });
      }
      continue;
    }
    list.push({
      id: cmd.id,
      title: cmd.title,
      shortcut: cmd.shortcut,
      category: cmd.category ?? "general",
    });
  }
  return list;
});

const filteredShortcuts = computed(() => {
  const needle = query.value.trim();
  if (!needle) return allShortcuts.value;
  return search(needle, [...allShortcuts.value], {
    keySelector: (shortcut) => `${shortcut.title} ${shortcut.shortcut}`,
    threshold: 0.6,
  });
});

const groupedSections = computed(() => {
  const groups = groupBy(filteredShortcuts.value, (shortcut) => shortcut.category);
  return CATEGORY_ORDER.filter((cat) => groups[cat]?.length).map((cat) => ({
    category: cat,
    title: t(CATEGORY_LOCALE_KEYS[cat]),
    shortcuts: groups[cat]!,
  }));
});
</script>

<template>
  <DialogRoot v-model:open="open">
    <DialogPortal>
      <DialogOverlay class="fixed inset-0 z-40 bg-surface-scrim" />
      <DialogContent
        class="fixed top-4 right-4 bottom-4 z-50 flex w-90 max-w-[calc(100vw-2rem)] flex-col overflow-hidden rounded-xl border border-stroke-default material-overlay outline-none select-none"
      >
        <div class="shrink-0 px-5 pt-4 pb-2">
          <div class="flex items-center justify-between">
            <DialogTitle class="text-title-sm font-semibold tracking-tight text-fg-primary">
              {{ t("shell.shortcuts.title") }}
            </DialogTitle>
            <DialogClose as-child>
              <UiIconButton :label="t('common.close')" size="sm">
                <X class="size-3.5" :stroke-width="1.6" />
              </UiIconButton>
            </DialogClose>
          </div>

          <UiSearchInput
            v-model="query"
            autofocus
            class="mt-3"
            :placeholder="t('shell.shortcuts.search')"
            @escape="open = false"
          />
        </div>

        <div
          class="flex-1 scrollbar-none space-y-4 overflow-y-auto px-5 pt-1 pb-5 [&::-webkit-scrollbar]:hidden"
        >
          <section v-for="section in groupedSections" :key="section.category">
            <h3 class="mt-3.5 mb-1.5 text-label-md font-semibold tracking-tight text-fg-primary first:mt-1">
              {{ section.title }}
            </h3>
            <div class="space-y-0.5">
              <div
                v-for="shortcut in section.shortcuts"
                :key="shortcut.id"
                class="flex h-7.5 items-center justify-between py-0.5 text-label-md"
              >
                <span class="truncate pr-2 font-normal text-fg-primary">{{ shortcut.title }}</span>
                <UiKbd :shortcut="shortcut.shortcut" variant="badge" class="shrink-0" />
              </div>
            </div>
          </section>

          <div v-if="!groupedSections.length" class="py-12 text-center text-label-md text-fg-tertiary">
            {{ t("shell.shortcuts.noMatches") }}
          </div>
        </div>
      </DialogContent>
    </DialogPortal>
  </DialogRoot>
</template>
