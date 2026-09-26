<script setup lang="ts">
import { Star } from "@lucide/vue";
import { orderBy } from "es-toolkit";
import { search } from "fast-fuzzy";
import { storeToRefs } from "pinia";
import { computed, nextTick, shallowRef, useTemplateRef } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import UiScrollArea from "../../../../shared/ui/UiScrollArea.vue";
import { spaceAssetSource } from "../../assets/node-preview-source";
import type { Space, SpaceLibraryScope } from "../../model/space";
import { useSpaceStore } from "../../session/space-store";
import { popupSpaceContextMenu, spaceCheckItem, spaceMenuItem } from "../../workspace/space-menu";
import { useSpaceManagementMenu } from "../../workspace/use-space-management-menu";

const store = useSpaceStore();
const management = useSpaceManagementMenu();
const { spaces, libraryScope, libraryLayout, libraryQuery } = storeToRefs(store);
const { locale, t } = useI18n();
const editingId = shallowRef<string>();
const draftTitle = shallowRef("");
const libraryRoot = useTemplateRef<HTMLElement>("libraryRoot");
const titleKeys: Record<SpaceLibraryScope, string> = {
  all: "spaces.space.sections.all",
  recent: "spaces.space.sections.recent",
  favorites: "spaces.space.sections.favorites",
};
const title = computed(() => t(titleKeys[libraryScope.value]));
const visibleSpaces = computed(() => {
  const needle = libraryQuery.value.trim();
  const available = spaces.value.filter((space) => space.kind === "space" && space.archivedAt === null);
  const scoped = libraryScope.value === "favorites" ? available.filter((space) => space.favorite) : available;
  const matched = needle ? search(needle, scoped, { keySelector: (space) => space.title }) : scoped;
  return orderBy(matched, [libraryScope.value === "all" ? "createdAt" : "lastOpenedAt"], ["desc"]);
});

const updatedAt = computed(() => new Intl.DateTimeFormat(locale.value, { dateStyle: "medium" }));

function formatUpdatedAt(timestamp: number) {
  return updatedAt.value.format(timestamp);
}

async function beginRenaming(space: Space) {
  editingId.value = space.id;
  draftTitle.value = space.title;
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
  <UiScrollArea class="h-full bg-canvas">
    <section ref="libraryRoot" class="px-6 py-5">
      <h1 class="text-display font-semibold text-accent">{{ title }}</h1>

      <div
        v-if="visibleSpaces.length"
        :class="
          libraryLayout === 'grid'
            ? 'mt-5 grid grid-cols-[repeat(auto-fill,minmax(210px,240px))] gap-5'
            : 'mt-4 divide-y divide-divider border-y border-divider'
        "
      >
        <article
          v-for="space in visibleSpaces"
          :key="space.id"
          class="group relative overflow-hidden bg-surface"
          :class="libraryLayout === 'grid' ? 'rounded-panel border border-border' : 'flex items-center py-2'"
          @contextmenu.prevent="showSpaceMenu(space)"
        >
          <button
            type="button"
            class="absolute inset-0 z-10 rounded-panel outline-none focus-visible:ring-2 focus-visible:ring-focus-ring focus-visible:ring-inset"
            @click="store.openSpace(space.id)"
          />
          <div
            class="shrink-0 overflow-hidden bg-canvas"
            :class="
              libraryLayout === 'grid'
                ? 'aspect-video w-full'
                : 'h-16 w-28 rounded-control border border-border'
            "
          >
            <img
              v-if="space.thumbnailPath"
              :src="spaceAssetSource(space.thumbnailPath)"
              alt=""
              class="size-full object-cover"
            />
          </div>
          <div
            class="relative min-w-0 flex-1 px-3 py-2.5"
            :class="libraryLayout === 'grid' ? 'border-t border-divider' : ''"
          >
            <UiIconButton
              v-if="space.kind === 'space'"
              class="absolute top-1.5 right-1.5 z-20 opacity-0 group-focus-within:opacity-100 group-hover:opacity-100"
              size="sm"
              :active="space.favorite"
              :label="t(space.favorite ? 'spaces.space.unfavorite' : 'spaces.space.favorite')"
              @click="store.setSpaceFavorite(space.id, !space.favorite)"
            >
              <Star class="size-(--knovum-icon-size-sm)" :fill="space.favorite ? 'currentColor' : 'none'" />
            </UiIconButton>
            <UiInput
              v-if="editingId === space.id"
              v-model="draftTitle"
              data-library-title-editor
              size="sm"
              class="relative z-20 w-full"
              @click.stop
              @blur="finishRenaming"
              @keydown.enter.prevent="finishRenaming"
              @keydown.escape.prevent="editingId = undefined"
            />
            <h2 v-else class="truncate pr-7 text-label font-semibold text-foreground">{{ space.title }}</h2>
            <p class="mt-0.5 text-caption text-foreground-tertiary">
              {{ t("spaces.space.nodeCount", { count: space.nodeCount }) }} ·
              {{ formatUpdatedAt(space.updatedAt) }}
            </p>
          </div>
        </article>
      </div>
      <div v-else class="grid min-h-64 place-items-center text-body text-foreground-tertiary">
        {{ t("spaces.space.search.noResults") }}
      </div>
    </section>
  </UiScrollArea>
</template>
