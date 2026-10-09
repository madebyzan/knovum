<script setup lang="ts">
import { refDebounced } from "@vueuse/core";
import { orderBy } from "es-toolkit";
import { search } from "fast-fuzzy";
import { storeToRefs } from "pinia";
import { computed, nextTick, shallowRef, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import UiScrollArea from "../../../../shared/ui/UiScrollArea.vue";
import type { Space, SpaceLibraryScope } from "../../model/space";
import { useSpaceStore } from "../../session/space-store";
import { popupSpaceContextMenu, spaceCheckItem, spaceMenuItem } from "../../workspace/space-menu";
import { useSpaceManagementMenu } from "../../workspace/use-space-management-menu";
import SpaceLibraryCard from "./SpaceLibraryCard.vue";
import { useLibraryViewport } from "./use-library-viewport";

const store = useSpaceStore();
const management = useSpaceManagementMenu();
const { spaces, libraryScope, libraryLayout, libraryQuery } = storeToRefs(store);
const { t } = useI18n();
const editingId = shallowRef<string>();
const draftTitle = shallowRef("");
const libraryRoot = useTemplateRef<HTMLElement>("libraryRoot");
const titleKeys: Record<SpaceLibraryScope, string> = {
  all: "spaces.space.sections.all",
  recent: "spaces.space.sections.recent",
  favorites: "spaces.space.sections.favorites",
};
const title = computed(() => t(titleKeys[libraryScope.value]));
const query = refDebounced(libraryQuery, 150);
const scrollArea = useTemplateRef<InstanceType<typeof UiScrollArea>>("scrollArea");
const rowsHost = useTemplateRef<HTMLElement>("rowsHost");
const orderedSpaces = computed(() => {
  const available = spaces.value.filter((space) => space.kind === "space" && space.archivedAt === null);
  const scoped = libraryScope.value === "favorites" ? available.filter((space) => space.favorite) : available;
  return orderBy(scoped, [libraryScope.value === "all" ? "createdAt" : "lastOpenedAt"], ["desc"]);
});
const visibleSpaces = computed(() => {
  const needle = query.value.trim();
  if (!needle) return orderedSpaces.value;
  const matched = new Set(
    search(needle, orderedSpaces.value, { keySelector: (space) => space.title }).map(({ id }) => id),
  );
  return orderedSpaces.value.filter(({ id }) => matched.has(id));
});

const viewport = useLibraryViewport({
  spaces: visibleSpaces,
  layout: libraryLayout,
  root: libraryRoot,
  host: rowsHost,
  scroll: () => scrollArea.value?.viewportElement,
  editingId,
});

watch([query, libraryScope], () => viewport.reset());

async function beginRenaming(space: Space) {
  editingId.value = space.id;
  draftTitle.value = space.title;
  await viewport.reveal(space.id);
  await nextTick();
  libraryRoot.value?.querySelector<HTMLInputElement>("[data-library-title-editor]")?.select();
}

function finishRenaming() {
  const id = editingId.value;
  const nextTitle = draftTitle.value.trim();
  if (id && nextTitle) void store.renameSpace(id, nextTitle);
  editingId.value = undefined;
}

async function archiveSpace(space: Space) {
  await store.archiveSpace(space.id);
  toast.success(t("spaces.space.archived", { title: space.title }), {
    action: { label: t("common.undo"), onClick: () => void store.unarchiveSpace(space.id) },
  });
}

function showSpaceMenu(space: Space) {
  if (space.kind === "inbox") return;
  void popupSpaceContextMenu([
    [spaceMenuItem(t("spaces.space.open"), () => store.openSpace(space.id))],
    [spaceMenuItem(t("spaces.space.rename"), () => beginRenaming(space))],
    [
      spaceCheckItem(t("spaces.space.favorite"), space.favorite, () =>
        store.setSpaceFavorite(space.id, !space.favorite),
      ),
    ],
    ...management.groups(space),
    [spaceMenuItem(t("spaces.space.archive"), () => archiveSpace(space))],
  ]);
}
</script>

<template>
  <UiScrollArea ref="scrollArea" class="h-full bg-surface-canvas">
    <section ref="libraryRoot" class="px-6 py-5">
      <h1 class="text-display font-semibold text-fg-accent">{{ title }}</h1>

      <div
        ref="rowsHost"
        v-if="visibleSpaces.length"
        class="relative mt-5"
        :style="{ height: `${viewport.height.value}px` }"
        @keydown="viewport.keydown"
      >
        <div
          v-for="row in viewport.rows.value"
          :key="row.index"
          :data-index="row.index"
          :ref="(element) => viewport.measure(element as HTMLElement | null)"
          class="absolute top-0 left-0 w-full"
          :class="libraryLayout === 'grid' ? 'grid gap-5 pb-5' : 'border-b border-stroke-divider'"
          :style="{
            transform: `translateY(${row.start - viewport.margin.value}px)`,
            gridTemplateColumns:
              libraryLayout === 'grid'
                ? `repeat(${viewport.columns.value}, minmax(210px, 240px))`
                : undefined,
          }"
        >
          <SpaceLibraryCard
            v-for="space in visibleSpaces.slice(
              row.index * viewport.columns.value,
              (row.index + 1) * viewport.columns.value,
            )"
            :key="space.id"
            :space="space"
            :layout="libraryLayout"
            :editing="editingId === space.id"
            :tab-index="viewport.focusedId.value === space.id ? 0 : -1"
            v-model:draft-title="draftTitle"
            @focusin="viewport.focus(space.id)"
            @open="store.openSpace(space.id)"
            @menu="showSpaceMenu(space)"
            @favorite="store.setSpaceFavorite(space.id, !space.favorite)"
            @finish="finishRenaming"
            @cancel="editingId = undefined"
          />
        </div>
      </div>
      <div v-else class="grid min-h-64 place-items-center text-body text-fg-tertiary">
        {{ t("spaces.space.search.noResults") }}
      </div>
    </section>
  </UiScrollArea>
</template>
