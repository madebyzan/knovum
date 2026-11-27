<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, shallowRef, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import { POPOVER_SURFACE_CLASSES } from "../../../../shared/ui/ui-classes";
import UiKbd from "../../../../shared/ui/UiKbd.vue";
import UiKnovumAiIcon from "../../../../shared/ui/UiKnovumAiIcon.vue";
import {
  SLASH_GROUP_LABEL_KEY,
  type SlashCommandGroup,
  type SlashCommandItem,
} from "../slash-command-definition";
import type { SlashCommandController } from "../tiptap-slash-command";

const props = defineProps<{ controller: SlashCommandController }>();
const { t } = useI18n();
const listRef = useTemplateRef<HTMLElement>("listRef");
const measuredHeight = shallowRef(260);
const state = computed(() => props.controller.state.value);

const MENU_WIDTH = 304;
const VIEWPORT_PADDING = 12;

const positionStyle = computed(() => {
  const rect = state.value.rect;
  if (!rect) return undefined;
  const maxLeft = Math.max(VIEWPORT_PADDING, globalThis.innerWidth - MENU_WIDTH - VIEWPORT_PADDING);
  const left = Math.min(Math.max(VIEWPORT_PADDING, rect.left), maxLeft);
  const height = Math.min(measuredHeight.value, 336);
  const fitsBelow = rect.bottom + 8 + height <= globalThis.innerHeight - VIEWPORT_PADDING;
  const top = fitsBelow ? rect.bottom + 8 : Math.max(VIEWPORT_PADDING, rect.top - height - 8);
  return { left: `${Math.round(left)}px`, top: `${Math.round(top)}px` };
});

interface GroupedSlashSection {
  readonly group: SlashCommandGroup;
  readonly entries: readonly { readonly item: SlashCommandItem; readonly index: number }[];
}

const sections = computed<readonly GroupedSlashSection[]>(() => {
  const map = new Map<SlashCommandGroup, { item: SlashCommandItem; index: number }[]>();
  state.value.items.forEach((item, index) => {
    const bucket = map.get(item.group) ?? [];
    bucket.push({ item, index });
    map.set(item.group, bucket);
  });
  return [...map.entries()].map(([group, entries]) => ({ group, entries }));
});

const handleViewportChange = () => props.controller.refreshPosition();

watch(
  [() => state.value.open, () => state.value.items.length],
  async ([isOpen]) => {
    if (isOpen) {
      globalThis.addEventListener("scroll", handleViewportChange, { capture: true, passive: true });
      globalThis.addEventListener("resize", handleViewportChange, { passive: true });
      await nextTick();
      if (listRef.value) measuredHeight.value = listRef.value.offsetHeight || 260;
    } else {
      globalThis.removeEventListener("scroll", handleViewportChange, true);
      globalThis.removeEventListener("resize", handleViewportChange);
    }
  },
  { immediate: true },
);

watch(
  () => state.value.selectedIndex,
  async (index) => {
    if (!state.value.open) return;
    await nextTick();
    listRef.value
      ?.querySelector<HTMLElement>(`[data-slash-index="${index}"]`)
      ?.scrollIntoView({ block: "nearest" });
  },
);

onBeforeUnmount(() => {
  globalThis.removeEventListener("scroll", handleViewportChange, true);
  globalThis.removeEventListener("resize", handleViewportChange);
});
</script>

<template>
  <Teleport to="body">
    <div
      v-if="state.open && positionStyle"
      ref="listRef"
      data-testid="tiptap-slash-menu"
      role="listbox"
      :aria-label="t('spaces.richtext.slash.title')"
      :style="positionStyle"
      :class="[POPOVER_SURFACE_CLASSES, 'fixed z-50 max-h-84 w-76 overflow-y-auto p-1.5 select-none']"
      @mousedown.prevent
    >
      <template v-if="sections.length">
        <div
          v-for="(section, sectionIdx) in sections"
          :key="section.group"
          :class="sectionIdx > 0 && 'mt-1.5 border-t border-stroke-divider pt-1.5'"
        >
          <div class="px-2 py-1 text-caption-sm font-medium text-fg-tertiary">
            {{ t(SLASH_GROUP_LABEL_KEY[section.group]) }}
          </div>
          <button
            v-for="{ item, index } in section.entries"
            :key="item.id"
            type="button"
            role="option"
            :aria-selected="index === state.selectedIndex"
            :data-slash-index="index"
            :data-highlighted="index === state.selectedIndex ? 'true' : undefined"
            class="flex w-full items-center gap-2.5 rounded-md px-2 py-1.5 text-left transition-colors duration-100 ease-standard outline-none data-[highlighted=true]:bg-surface-control-hover"
            @mouseenter="controller.selectIndex(index)"
            @click="controller.executeItem(item)"
          >
            <span
              class="grid size-8 shrink-0 place-items-center rounded-md border border-stroke-default bg-surface-raised"
            >
              <UiKnovumAiIcon v-if="item.id === 'ai-ask'" class="size-4 text-fg-primary" />
              <component :is="item.icon" v-else class="size-4 text-fg-secondary" :stroke-width="1.6" />
            </span>
            <span class="min-w-0 flex-1">
              <span class="block truncate text-label font-medium text-fg-primary">{{
                t(item.labelKey)
              }}</span>
              <span class="block truncate text-caption-sm text-fg-tertiary">{{
                t(item.descriptionKey)
              }}</span>
            </span>
            <UiKbd v-if="item.shortcut" class="shrink-0 font-mono">{{ item.shortcut }}</UiKbd>
          </button>
        </div>
      </template>
      <p v-else class="px-2.5 py-4 text-center text-caption text-fg-tertiary">
        {{ t("spaces.richtext.slash.empty") }}
      </p>
    </div>
  </Teleport>
</template>
