<script setup lang="ts">
import { Tag, X } from "@lucide/vue";
import { compact, countBy, take, uniq } from "es-toolkit";
import { search } from "fast-fuzzy";
import { storeToRefs } from "pinia";
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";

import UiBadge from "../../../shared/ui/UiBadge.vue";
import UiInput from "../../../shared/ui/UiInput.vue";
import { useCanvasEditorSession, useCanvasSelectionProjection } from "../canvas/editor-session";
import { useSpaceStore } from "../session/space-store";
import { popupSpaceContextMenu, spaceMenuItem } from "../workspace/space-menu";

withDefaults(defineProps<{ mode?: "both" | "browse" | "selection" }>(), { mode: "both" });
const store = useSpaceStore();
const { nodes } = storeToRefs(store);
const editorSession = useCanvasEditorSession();
const { selectedNodes } = useCanvasSelectionProjection(nodes);
const { t } = useI18n();
const draft = ref("");
const selectedItems = computed(() => selectedNodes.value.filter((node) => node.data.kind !== "section"));
const selectedIds = computed(() => selectedItems.value.map((node) => node.id));
const selectionTagCounts = computed(
  () =>
    new Map(
      Object.entries(
        countBy(
          selectedItems.value.flatMap((node) => node.data.tags ?? []),
          (tag) => tag,
        ),
      ),
    ),
);
const tagCounts = computed(() =>
  Object.entries(
    countBy(
      nodes.value.filter((node) => node.data.kind !== "section").flatMap((node) => node.data.tags ?? []),
      (tag) => tag,
    ),
  ).toSorted(([left], [right]) => left.localeCompare(right)),
);
const suggestions = computed(() => {
  const query = draft.value.trim();
  if (!query) return [];
  const candidates = tagCounts.value.filter(([tag]) => !selectionTagCounts.value.has(tag));
  return take(search(query, candidates, { keySelector: ([tag]) => tag }), 5);
});

async function applyTag(tag = draft.value) {
  const value = tag.trim();
  const ids = [...selectedIds.value];
  const spaceId = store.activeSpaceId;
  const initialDraft = draft.value;
  if (!value || ids.length === 0 || !spaceId) return;
  await store.addTag(ids, value);
  if (store.activeSpaceId === spaceId && draft.value === initialDraft) draft.value = "";
}
async function revealTag(tag: string) {
  const spaceId = store.activeSpaceId;
  if (!spaceId) return;
  const matches = nodes.value.filter((node) => node.data.kind !== "section" && node.data.tags?.includes(tag));
  const sectionIds = uniq(compact(matches.map((node) => node.data.sectionId)));
  await Promise.all(
    sectionIds.map((id) => {
      const section = nodes.value.find((node) => node.id === id);
      return section?.data.kind === "section" && section.data.collapsed
        ? store.setSectionCollapsed(id, false)
        : Promise.resolve();
    }),
  );
  if (store.activeSpaceId !== spaceId) return;
  const ids = matches.map((node) => node.id);
  editorSession.reveal(ids, "focus");
}
async function showTagMenu(tag: string) {
  await popupSpaceContextMenu([
    [spaceMenuItem(t("spaces.navigation.showTaggedItems"), () => revealTag(tag))],
  ]);
}
</script>

<template>
  <section v-if="mode !== 'selection'">
    <h3 class="px-1 text-caption font-semibold text-foreground-secondary">
      {{ t("spaces.navigation.tags") }}
    </h3>
    <div v-if="tagCounts.length" class="mt-2 space-y-0.5">
      <button
        v-for="[tag, count] in tagCounts"
        :key="tag"
        type="button"
        class="flex h-(--knovum-list-item-height) w-full items-center gap-2 rounded-control px-2 text-left text-caption text-foreground-secondary hover:bg-control-hover hover:text-foreground"
        @click="revealTag(tag)"
        @contextmenu.prevent="showTagMenu(tag)"
      >
        <Tag class="size-(--knovum-icon-size-xs) shrink-0 text-foreground-tertiary" :stroke-width="1.6" />
        <span class="min-w-0 flex-1 truncate">{{ tag }}</span>
        <UiBadge size="sm">{{ count }}</UiBadge>
      </button>
    </div>
    <p v-else class="mt-2 px-1 text-caption text-foreground-tertiary">
      {{ t("spaces.navigation.tagsEmpty") }}
    </p>
  </section>

  <section v-if="mode !== 'browse'">
    <div v-if="selectedIds.length === 0" class="px-1 text-caption text-foreground-tertiary">
      {{ t("spaces.navigation.tagsSelectionHint") }}
    </div>
    <template v-else>
      <div class="relative">
        <UiInput
          v-model="draft"
          class="w-full"
          :placeholder="t('spaces.navigation.addTag')"
          @keydown.enter="applyTag()"
        />
        <div
          v-if="suggestions.length"
          class="absolute top-full left-0 z-20 mt-1 w-full rounded-panel bg-overlay p-1 shadow-popover ring-1 ring-border"
        >
          <button
            v-for="[tag, count] in suggestions"
            :key="tag"
            type="button"
            class="flex h-(--knovum-compact-item-height) w-full items-center justify-between rounded-control px-2 text-left text-caption hover:bg-control-hover"
            @click="applyTag(tag)"
          >
            <span class="truncate">{{ tag }}</span>
            <span class="text-micro text-foreground-tertiary">{{ count }}</span>
          </button>
        </div>
      </div>
      <div v-if="selectionTagCounts.size" class="mt-2 flex flex-wrap gap-1.5">
        <span
          v-for="[tag, count] in selectionTagCounts"
          :key="tag"
          class="inline-flex h-(--knovum-control-height-sm) items-center gap-1 rounded-control bg-control pr-1 pl-2 text-caption text-foreground-secondary"
        >
          {{ tag }}
          <span v-if="count !== selectedIds.length" class="text-micro text-foreground-tertiary"
            >{{ count }}/{{ selectedIds.length }}</span
          >
          <button
            type="button"
            class="grid size-5 place-items-center rounded-control hover:bg-control-hover"
            :aria-label="t('spaces.navigation.removeTag', { tag })"
            @click="store.removeTag(selectedIds, tag)"
          >
            <X class="size-(--knovum-icon-size-xs)" />
          </button>
        </span>
      </div>
    </template>
  </section>
</template>
